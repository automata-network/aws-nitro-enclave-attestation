//! Debug utilities for inspecting and analyzing Nitro Enclave attestation reports.
//!
//! This module provides tools for examining the contents of attestation reports,
//! including the attestation document, certificate chain, and other metadata.

use std::path::PathBuf;

use alloy_primitives::Bytes;
use aws_nitro_enclave_attestation_verifier::{stub::Bytes48, AttestationReport};
use clap::{Args, Subcommand};
use x509_verifier_rust_crypto::x509_parser::time::ASN1Time;

/// Debug subcommands for attestation report analysis.
#[derive(Subcommand)]
pub enum DebugCli {
    /// Inspect and display attestation document contents
    Doc(DebugDocCli),
}

impl DebugCli {
    /// Executes the appropriate debug subcommand.
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            DebugCli::Doc(cli) => cli.run(),
        }
    }
}

/// Arguments for debugging attestation document contents.
#[derive(Args)]
pub struct DebugDocCli {
    /// Path to the Nitro Enclave attestation report file
    #[clap(long)]
    report: PathBuf,
}

impl DebugDocCli {
    /// Executes attestation document inspection and display.
    ///
    /// This method parses the attestation report and displays detailed information
    /// about the attestation document and certificate chain, including:
    /// - Module ID and timestamp
    /// - PCR values (Platform Configuration Registers)
    /// - Public key, user data, and nonce (if present)
    /// - Certificate chain information and validity periods
    pub fn run(&self) -> anyhow::Result<()> {
        // Parse the attestation report from file
        let report = AttestationReport::parse(&std::fs::read(&self.report)?)?;
        let cert_chain = report.cert_chain()?;
        let doc = report.doc();

        // Display attestation document information
        tracing::info!("Doc:");
        tracing::info!("\tModule ID: {}", doc.module_id);

        // Convert and display timestamp in human-readable format
        let timestamp = ASN1Time::from_timestamp(doc.timestamp as i64 / 1000)?;
        tracing::info!("\tTimestamp: {}({})", timestamp, timestamp.timestamp());
        tracing::info!("\tDigest: {}", doc.digest);

        // Display optional fields if present
        if let Some(data) = &doc.public_key {
            tracing::info!("\tPublicKey: {}", Bytes::copy_from_slice(data));
        }
        if let Some(data) = &doc.user_data {
            tracing::info!("\tUserData: {}", Bytes::copy_from_slice(data));
        }
        if let Some(data) = &doc.nonce {
            tracing::info!("\tNonce: {}", Bytes::copy_from_slice(data));
        }

        // Display non-zero PCR values
        for (k, v) in &doc.pcrs {
            let v = Bytes48::from(v);
            if v.is_zero() {
                continue;
            }
            tracing::info!("\tPCR[{}]: {}", k, v);
        }

        // Display certificate chain information
        tracing::info!("Cert Chain:");
        let digest = cert_chain.digest();
        for (idx, cert) in cert_chain.certs.iter().enumerate() {
            tracing::info!("\t[{idx}] Digest: {:?}", digest[idx]);
            let (start, end) = cert.validity();
            tracing::info!(
                "\t    Valid: {start}({}) - {end}({})",
                start.timestamp(),
                end.timestamp()
            );
        }
        Ok(())
    }
}
