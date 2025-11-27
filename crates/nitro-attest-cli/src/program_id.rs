//! Program ID retrieval functionality.
//!
//! This module provides functionality to retrieve the program identifiers
//! for the verifier and aggregator ZK programs.

use clap::Args;

use crate::utils::ProverArgs;

/// Command-line arguments for the program-id subcommand.
///
/// Retrieves and displays the program identifiers for the configured ZK prover.
#[derive(Args)]
pub struct ProgramIdCli {
    /// Zero-knowledge proof system configuration
    #[clap(flatten)]
    prover: ProverArgs,
}

impl ProgramIdCli {
    /// Executes the program-id command.
    ///
    /// This method creates a prover instance and retrieves the program IDs
    /// for both the verifier and aggregator programs.
    pub fn run(&self) -> anyhow::Result<()> {
        // Create the prover instance with the specified configuration
        let prover = self.prover.new_prover(None)?;

        // Get the program IDs
        let program_id = prover.get_program_id();

        // Display program ID information
        println!("Program IDs:");
        println!("  Verifier ID: {}", program_id.verifier_id);
        println!("  Verifier Proof ID: {}", program_id.verifier_proof_id);
        println!("  Aggregator ID: {}", program_id.aggregator_id);

        Ok(())
    }
}
