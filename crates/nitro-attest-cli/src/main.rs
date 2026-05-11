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
mod program_id;
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

    /// Get the program IDs for the configured ZK prover
    ProgramId(program_id::ProgramIdCli),
}

/// Returns true if every active zkVM's embedded ELFs were built reproducibly.
/// Any false flips the whole binary's state; an aggregated banner is shown once.
fn check_reproducible_build() -> bool {
    #[allow(unused_mut)]
    let mut reproducible = true;

    #[cfg(feature = "sp1")]
    {
        reproducible &= aws_nitro_enclave_attestation_prover::SP1_REPRODUCIBLE_BUILD;
    }
    #[cfg(feature = "risc0")]
    {
        reproducible &= aws_nitro_enclave_attestation_prover::RISC0_REPRODUCIBLE_BUILD;
    }

    reproducible
}

/// Prints a multi-line warning banner to stderr. Uses stderr (not tracing) so
/// it cannot be silenced by `RUST_LOG=...` filters.
fn print_non_reproducible_warning_banner() {
    let lines = [
        "================================================================",
        "  WARNING: This binary was built WITHOUT reproducible build.",
        "",
        "  ZK program IDs may differ from the official release.",
        "  DO NOT use this binary to generate production proofs.",
        "",
        "  To enable reproducible build, unset REPRODUCIBLE_BUILD",
        "  (or set REPRODUCIBLE_BUILD=1) and rebuild.",
        "================================================================",
    ];
    for line in lines {
        eprintln!("{line}");
    }
}

fn main() -> anyhow::Result<()> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    if !check_reproducible_build() {
        print_non_reproducible_warning_banner();
    }

    let cli = NitroAttestCli::parse();
    match &cli.command {
        Commands::Prove(cli) => cli.run()?,
        Commands::Debug(cli) => cli.run()?,
        Commands::Upload(cli) => cli.run()?,
        Commands::Proof(cli) => cli.run()?,
        Commands::ProgramId(cli) => cli.run()?,
    }
    Ok(())
}
