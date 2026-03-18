//! Proof generation functionality for Nitro Enclave attestation reports.
//!
//! This module handles the creation of zero-knowledge proofs from AWS Nitro Enclave
//! attestation reports using either RISC0 or SP1 proof systems.

use std::path::PathBuf;

use anyhow::anyhow;
use aws_nitro_enclave_attestation_prover::set_prover_dev_mode;
use clap::Args;

use crate::utils::{ContractArgs, ProverArgs};

/// Command-line arguments for the prove subcommand.
///
/// Generates zero-knowledge proofs from one or more Nitro Enclave attestation reports.
/// Supports both single report verification and multi-report aggregation.
#[derive(Args)]
pub struct ProveCli {
    /// Path(s) to Nitro Enclave attestation report files
    ///
    /// Can specify multiple report files to generate an aggregated proof.
    /// Each file should contain a binary attestation report from AWS Nitro Enclaves.
    #[arg(long)]
    report: Vec<PathBuf>,

    /// Output file path for the generated proof
    ///
    /// If not specified, the proof will only be printed to stdout.
    /// The output format is JSON containing the proof data and metadata.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Zero-knowledge proof system configuration
    #[clap(flatten)]
    prover: ProverArgs,

    /// Smart contract configuration for on-chain verification
    #[clap(flatten)]
    contract: ContractArgs,
}

impl ProveCli {
    /// Executes the proof generation command.
    ///
    /// This method orchestrates the entire proof generation process:
    /// 1. Configures the prover with development mode settings
    /// 2. Validates input parameters
    /// 3. Reads attestation report files
    /// 4. Creates the appropriate prover instance
    /// 5. Generates proofs (single or aggregated)
    /// 6. Outputs results to file and/or stdout
    pub fn run(&self) -> anyhow::Result<()> {
        set_prover_dev_mode(self.prover.dev);
        if self.report.len() == 0 {
            return Err(anyhow!(
                "No report files provided. Use --report to specify the report files."
            ));
        }

        let mut raw_reports = Vec::with_capacity(self.report.len());
        for report in &self.report {
            raw_reports.push(std::fs::read(report)?);
        }

        // Initialize smart contract interface (if configured)
        let contract = self.contract.stub()?;

        // Create the prover instance with the specified configuration
        let prover = self.prover.new_prover(contract)?;

        // Generate proof based on the number of input reports
        let result = if raw_reports.len() == 1 {
            prover.prove_attestation_report(raw_reports.remove(0))?
        } else {
            prover.prove_multiple_reports(raw_reports)?
        };

        // Write proof to output file if specified
        if let Some(out) = &self.out {
            std::fs::write(out, result.encode_json()?)?;
        }

        // Display proof information to stdout
        println!("proof: {:?}", result);

        Ok(())
    }
}
