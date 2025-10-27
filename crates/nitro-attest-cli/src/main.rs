//! # Nitro Attestation CLI
//!
//! A command-line interface for generating and verifying AWS Nitro Enclave attestation proofs
//! using zero-knowledge proof systems (RISC0, SP1, and Pico).
//!
//! This CLI provides functionality to:
//! - Generate ZK proofs for Nitro Enclave attestation reports
//! - Verify proofs on-chain using smart contracts
//! - Aggregate multiple proofs together
//! - Upload ZK programs for remote execution
//! - Debug and inspect attestation reports
//!
//! ## Examples
//!
//! Generate a proof from an attestation report:
//! ```bash
//! nitro-attest-cli prove --report attestation.report --sp1 --out proof.json
//! # or use --risc0 or --pico
//! ```
//!
//! Verify a proof on-chain:
//! ```bash
//! nitro-attest-cli proof verify-on-chain --proof proof.json --contract 0x... --rpc-url https://...
//! ```

use clap::{Parser, Subcommand};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

mod debug;
mod proof;
mod prove;
mod upload;
mod utils;

/// Main CLI application structure for Nitro Attestation CLI
#[derive(Parser)]
#[command(name = "nitro-attest-cli")]
#[command(version)]
#[command(about = "CLI for AWS Nitro Enclave attestation proof generation and verification")]
struct NitroAttestCli {
    #[command(subcommand)]
    command: Commands,
}

/// Available subcommands for the CLI
#[derive(Subcommand)]
enum Commands {
    /// Generate zero-knowledge proofs from Nitro Enclave attestation reports
    Prove(prove::ProveCli),
    
    /// Proof-related operations (verification, aggregation, etc.)
    #[command(subcommand)]
    Proof(proof::ProofCli),
    
    /// Upload ZK programs for remote execution
    Upload(upload::UploadCli),
    
    /// Debug utilities for inspecting attestation reports
    #[command(subcommand)]
    Debug(debug::DebugCli),
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let cli = NitroAttestCli::parse();
    match &cli.command {
        Commands::Prove(cli) => cli.run()?,
        Commands::Debug(cli) => cli.run()?,
        Commands::Upload(cli) => cli.run()?,
        Commands::Proof(cli) => cli.run()?,
    }
    Ok(())
}
