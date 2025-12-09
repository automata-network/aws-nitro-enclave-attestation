//! Utility modules for CLI argument parsing and configuration.
//!
//! This module contains shared argument structures and helper functions
//! used across different CLI commands for configuring provers and smart contracts.

use alloy_primitives::Address;
use anyhow::{anyhow, bail};
use aws_nitro_enclave_attestation_prover::{
    NitroEnclaveProver, NitroEnclaveVerifierContract, ProverConfig,
};
use clap::{Args, ValueEnum};

/// Proof type for Boundless proving (CLI enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum BoundlessProofTypeCli {
    #[default]
    #[value(name = "groth16")]
    Groth16,
    #[value(name = "merkle")]
    Merkle,
}

/// Command-line arguments for configuring zero-knowledge proof system settings.
///
/// Supports RISC0, SP1, and Pico proof systems with their respective configuration options.
/// Only one prover type should be specified at a time.
#[derive(Args, Clone)]
pub struct ProverArgs {
    #[cfg(feature = "risc0")]
    /// Use the RISC0 zkVM for proof generation
    #[arg(long)]
    pub risc0: bool,

    #[cfg(feature = "sp1")]
    /// Use the SP1 zkVM for proof generation
    #[arg(long)]
    pub sp1: bool,

    #[cfg(feature = "pico")]
    /// Use the Pico zkVM for proof generation
    #[arg(long)]
    pub pico: bool,

    /// Enable development mode for mock proof generation
    #[arg(long, default_value = "false", env = "DEV_MODE")]
    pub dev: bool,

    /// Private key for SP1 network prover
    #[arg(long, env = "NETWORK_PRIVATE_KEY")]
    pub sp1_private_key: Option<String>,

    /// RPC URL for SP1 network connection
    #[arg(long)]
    pub sp1_rpc_url: Option<String>,

    /// Boundless RPC URL for RISC0 proving
    #[arg(long, env = "BOUNDLESS_RPC_URL")]
    pub risc0_rpc_url: Option<String>,

    /// Boundless wallet private key (hex-encoded)
    #[arg(long, env = "BOUNDLESS_PRIVATE_KEY")]
    pub risc0_private_key: Option<String>,

    /// Verifier program URL for pre-uploaded ELF (optional, uploads to IPFS if not set)
    #[arg(long, env = "BOUNDLESS_VERIFIER_PROGRAM_URL")]
    pub risc0_verifier_program_url: Option<String>,

    /// Aggregator program URL for pre-uploaded ELF (optional, uploads to IPFS if not set)
    #[arg(long, env = "BOUNDLESS_AGGREGATOR_PROGRAM_URL")]
    pub risc0_aggregator_program_url: Option<String>,

    /// Proof type for Boundless proving (groth16 or merkle)
    #[arg(long, value_enum, default_value = "groth16")]
    pub risc0_proof_type: BoundlessProofTypeCli,

    /// Minimum price in wei per cycle
    #[arg(long, env = "BOUNDLESS_MIN_PRICE")]
    pub risc0_min_price: Option<u128>,

    /// Maximum price in wei per cycle
    #[arg(long, env = "BOUNDLESS_MAX_PRICE")]
    pub risc0_max_price: Option<u128>,

    /// Timeout in seconds
    #[arg(long, env = "BOUNDLESS_TIMEOUT")]
    pub risc0_timeout: Option<u32>,

    /// Ramp-up period in seconds
    #[arg(long, env = "BOUNDLESS_RAMP_UP_PERIOD")]
    pub risc0_ramp_up_period: Option<u32>,
}

impl ProverArgs {
    /// Creates a prover configuration based on the specified arguments.
    pub fn prover_config(&self) -> anyhow::Result<ProverConfig> {
        // Check for mutual exclusion of prover options
        let prover_count = {
            let mut count = 0;
            #[cfg(feature = "risc0")]
            if self.risc0 { count += 1; }
            #[cfg(feature = "sp1")]
            if self.sp1 { count += 1; }
            #[cfg(feature = "pico")]
            if self.pico { count += 1; }
            count
        };

        if prover_count > 1 {
            return Err(anyhow!(
                "Cannot use multiple provers at the same time. Choose only one: --risc0, --sp1, or --pico"
            ));
        }

        #[cfg(feature = "sp1")]
        if self.sp1 {
            use aws_nitro_enclave_attestation_prover::SP1ProverConfig;
            return Ok(ProverConfig::sp1_with(SP1ProverConfig {
                private_key: self.sp1_private_key.clone(),
                rpc_url: self.sp1_rpc_url.clone(),
            }));
        }

        #[cfg(feature = "risc0")]
        if self.risc0 {
            use aws_nitro_enclave_attestation_prover::{
                program_risc0::BoundlessProofType, RiscZeroProverConfig,
            };

            let proof_type = match self.risc0_proof_type {
                BoundlessProofTypeCli::Merkle => BoundlessProofType::Merkle,
                BoundlessProofTypeCli::Groth16 => BoundlessProofType::Groth16,
            };

            return Ok(ProverConfig::risc0_with(RiscZeroProverConfig {
                rpc_url: self.risc0_rpc_url.clone(),
                private_key: self.risc0_private_key.clone(),
                verifier_program_url: self.risc0_verifier_program_url.clone(),
                aggregator_program_url: self.risc0_aggregator_program_url.clone(),
                proof_type,
                min_price: self.risc0_min_price,
                max_price: self.risc0_max_price,
                timeout: self.risc0_timeout,
                ramp_up_period: self.risc0_ramp_up_period,
            }));
        }

        #[cfg(feature = "pico")]
        if self.pico {
            use aws_nitro_enclave_attestation_prover::PicoProverConfig;
            return Ok(ProverConfig::pico_with(PicoProverConfig::default()));
        }

        bail!("No prover specified. Use --risc0, --sp1, or --pico to select a proof system.");
    }

    /// Creates a new `NitroEnclaveProver` instance with the configured settings.
    pub fn new_prover(
        &self,
        contract: Option<NitroEnclaveVerifierContract>,
    ) -> anyhow::Result<NitroEnclaveProver> {
        Ok(NitroEnclaveProver::new(self.prover_config()?, contract))
    }
}

/// Command-line arguments for configuring smart contract interaction.
/// 
/// Used for on-chain proof verification and other blockchain operations.
#[derive(Args, Clone)]
pub struct ContractArgs {
    /// The address of the Nitro Enclave Verifier contract
    #[arg(long, env = "CONTRACT")]
    pub contract: Option<Address>,

    /// The RPC URL to connect to the Ethereum network
    #[arg(long, env = "RPC_URL", default_value = "http://localhost:8545")]
    pub rpc_url: Option<String>,
}

impl ContractArgs {
    /// Checks if the contract configuration is incomplete.
    pub fn empty(&self) -> bool {
        self.contract.is_none() || self.rpc_url.is_none()
    }

    /// Creates a contract interface if all required parameters are provided.
    pub fn stub(&self) -> anyhow::Result<Option<NitroEnclaveVerifierContract>> {
        if self.empty() {
            return Ok(None);
        }
        let contract = *self.contract.as_ref().unwrap();
        let rpc_url = self.rpc_url.as_ref().unwrap();
        let verifier = NitroEnclaveVerifierContract::dial(&rpc_url, contract, None)?;
        Ok(Some(verifier))
    }
}
