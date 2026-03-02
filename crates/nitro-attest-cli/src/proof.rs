//! Proof management and verification operations.
//!
//! This module provides functionality for working with generated proofs including
//! on-chain verification, proof aggregation, and composite proof generation.

use std::path::PathBuf;

use anyhow::anyhow;
use aws_nitro_enclave_attestation_prover::{
    set_prover_dev_mode, utils::block_on, OnchainProof, ProofType,
};
use clap::{Args, Subcommand};

use crate::utils::{ContractArgs, ProverArgs};

/// Subcommands for proof-related operations.
#[derive(Subcommand)]
pub enum ProofCli {
    /// Verify a proof on-chain using smart contract
    VerifyOnChain(ProofVerifyOnChainCli),

    /// Generate composite proofs for single attestation reports  
    GenComposite(ProofGenCompositeCli),

    /// Aggregate multiple proofs into a single proof
    Aggregate(ProofAggregateCli),
}

impl ProofCli {
    /// Executes the appropriate proof subcommand.
    pub fn run(&self) -> anyhow::Result<()> {
        match self {
            ProofCli::VerifyOnChain(cli) => cli.run(),
            ProofCli::Aggregate(cli) => cli.run(),
            ProofCli::GenComposite(cli) => cli.run(),
        }
    }
}

/// Arguments for verifying proofs on-chain through smart contracts.
#[derive(Args)]
pub struct ProofVerifyOnChainCli {
    /// Path to the proof file to verify
    #[clap(long)]
    proof: PathBuf,

    /// Smart contract configuration for verification
    #[clap(flatten)]
    contract: ContractArgs,
}

impl ProofVerifyOnChainCli {
    /// Executes on-chain proof verification.
    ///
    /// This method submits a proof to the smart contract for verification,
    /// ensuring the proof was generated correctly and corresponds to valid
    /// Nitro Enclave attestation data.
    pub fn run(&self) -> anyhow::Result<()> {
        // Ensure contract configuration is provided
        let contract = self.contract.stub()?.ok_or_else(|| {
            anyhow!("No contract specified. Use --contract, --rpc-url to specify the contract.")
        })?;

        // Load and parse the proof file
        let result = OnchainProof::decode_json(&std::fs::read(&self.proof)?)?;

        // Validate that the proof contains on-chain verification data
        if result.onchain_proof.len() == 0 {
            return Err(anyhow::anyhow!(
                "Proof does not contain an on-chain proof, unable to submit."
            ));
        }

        // Verify proof to contract for verification
        let result = block_on(contract.verify_proof(&result))?;
        dbg!(result);

        Ok(())
    }
}

/// Arguments for aggregating multiple proofs into a single proof.
#[derive(Args)]
pub struct ProofAggregateCli {
    /// Paths to proof files to aggregate
    #[arg(long)]
    proof: Vec<PathBuf>,

    /// Output file path for the aggregated proof
    #[arg(long)]
    out: Option<PathBuf>,

    /// Smart contract configuration
    #[clap(flatten)]
    contract: ContractArgs,

    /// Zero-knowledge proof system configuration
    #[clap(flatten)]
    prover: ProverArgs,
}

impl ProofAggregateCli {
    /// Executes proof aggregation.
    ///
    /// Combines multiple individual proofs into a single aggregated proof,
    /// enabling efficient batch verification of multiple attestation reports.
    pub fn run(&self) -> anyhow::Result<()> {
        set_prover_dev_mode(self.prover.dev);

        // Validate that proof files are provided
        if self.proof.is_empty() {
            return Err(anyhow!(
                "No proof files provided. Use --proof to specify the proof files."
            ));
        }

        // Load and extract raw proofs from all proof files
        let mut proofs = Vec::with_capacity(self.proof.len());
        for proof_file in &self.proof {
            let proof = OnchainProof::decode_json(&std::fs::read(proof_file)?)?;
            proofs.push(proof.raw_proof);
        }

        // Initialize prover and contract interface
        let contract = self.contract.stub()?;
        let prover = self.prover.new_prover(contract)?;

        // Aggregate the proofs into a single proof
        let aggregated_proof = prover.aggregate_proofs(proofs)?;
        let aggregated_proof =
            prover.create_onchain_proof(aggregated_proof, ProofType::Aggregator)?;

        // Save aggregated proof to file if specified
        if let Some(out) = &self.out {
            std::fs::write(out, aggregated_proof.encode_json()?)?;
        }
        println!("proof: {:?}", aggregated_proof);

        Ok(())
    }
}

/// Arguments for generating composite proofs from attestation reports. Composite proofs will used for batch verification.
#[derive(Args)]
pub struct ProofGenCompositeCli {
    /// Path to the Nitro Enclave attestation report file
    #[arg(long)]
    report: PathBuf,

    /// Output file path for the composite proof
    #[arg(long)]
    out: Option<PathBuf>,

    /// Smart contract configuration
    #[clap(flatten)]
    contract: ContractArgs,

    /// Zero-knowledge proof system configuration
    #[clap(flatten)]
    prover: ProverArgs,
}

impl ProofGenCompositeCli {
    /// Executes composite proof generation.
    ///
    /// Creates a composite proof structure that can be used for more
    /// complex verification scenarios or as input to aggregation.
    pub fn run(&self) -> anyhow::Result<()> {
        set_prover_dev_mode(self.prover.dev);

        // Read the attestation report file
        let raw_report = std::fs::read(&self.report)?;

        // Initialize prover and contract interface
        let contract = self.contract.stub()?;
        let prover = self.prover.new_prover(contract)?;

        // Prepare inputs and generate composite proof
        let inputs = prover.prepare_verifier_inputs(vec![raw_report])?;
        let composite_proof = prover.gen_multi_composite_proofs(&inputs)?.remove(0);
        let composite_proof = prover.create_onchain_proof(composite_proof, ProofType::Verifier)?;

        // Save composite proof to file if specified
        if let Some(out) = &self.out {
            std::fs::write(out, composite_proof.encode_json()?)?;
        }
        println!("proof: {:?}", composite_proof);

        Ok(())
    }
}
