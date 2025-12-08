use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    program::{Program, RemoteProverConfig},
    utils::{block_on, parallels_blocking},
    NitroEnclaveVerifierContract, OnchainProof, OnchainProofVerifyResult, ProgramId, ProofType,
    RawProof, RawProofType,
};
use alloy_primitives::Bytes;
use anyhow::{anyhow, bail, Context};
use aws_nitro_enclave_attestation_verifier::{
    stub::{
        BatchVerifierInput, BatchVerifierJournal, VerifierInput, VerifierJournal, ZkCoProcessorType,
    },
    AttestationReport,
};

/// Configuration enumeration for different zero-knowledge proof systems.
///
/// This enum allows users to select and configure which ZK proof system
/// to use for generating proofs of AWS Nitro Enclave attestations.
/// Each variant corresponds to a different zkVM implementation with
/// its own performance characteristics and features.
///
/// # Available Backends
///
/// - **SP1 (Succinct)**: High-performance zkVM with network proving support
/// - **RISC0**: Industrial-grade zkVM with Bonsai cloud proving
///
/// # Feature Flags
///
/// The availability of each variant depends on compile-time feature flags:
/// - `sp1` feature enables the Succinct variant
/// - `risc0` feature enables the RiscZero variant
#[derive(Debug, Clone)]
pub struct ProverConfig {
    pub default_trusted_certs_prefix_length: u8,
    pub skip_time_validity_check: bool,
    pub skip_contract_program_id_check: bool,
    pub system: ProverSystemConfig,
}

impl ProverConfig {
    #[cfg(feature = "risc0")]
    pub fn risc0() -> Self {
        Self::risc0_with(Default::default())
    }

    #[cfg(feature = "risc0")]
    pub fn risc0_with(cfg: crate::program_risc0::RiscZeroProverConfig) -> Self {
        Self {
            default_trusted_certs_prefix_length: Self::default_trusted_certs_prefix_length(),
            skip_time_validity_check: Self::skip_time_validity_check(),
            skip_contract_program_id_check: Self::skip_contract_program_id_check(),
            system: ProverSystemConfig::RiscZero(cfg),
        }
    }

    #[cfg(feature = "sp1")]
    pub fn sp1() -> Self {
        Self::sp1_with(Default::default())
    }

    #[cfg(feature = "sp1")]
    pub fn sp1_with(cfg: crate::program_sp1::SP1ProverConfig) -> Self {
        Self {
            default_trusted_certs_prefix_length: Self::default_trusted_certs_prefix_length(),
            skip_time_validity_check: Self::skip_time_validity_check(),
            skip_contract_program_id_check: Self::skip_contract_program_id_check(),
            system: ProverSystemConfig::Succinct(cfg),
        }
    }

    #[cfg(feature = "pico")]
    pub fn pico() -> Self {
        Self::pico_with(Default::default())
    }

    #[cfg(feature = "pico")]
    pub fn pico_with(cfg: crate::program_pico::PicoProverConfig) -> Self {
        Self {
            default_trusted_certs_prefix_length: Self::default_trusted_certs_prefix_length(),
            skip_time_validity_check: Self::skip_time_validity_check(),
            skip_contract_program_id_check: Self::skip_contract_program_id_check(),
            system: ProverSystemConfig::Pico(cfg),
        }
    }

    fn default_trusted_certs_prefix_length() -> u8 {
        std::env::var("DEFAULT_TRUSTED_CERTS_PREFIX_LENGTH")
            .ok()
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(1_u8)
    }

    fn skip_time_validity_check() -> bool {
        std::env::var("SKIP_TIME_VALIDITY_CHECK")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false)
    }

    fn skip_contract_program_id_check() -> bool {
        std::env::var("SKIP_CONTRACT_PROGRAM_ID_CHECK")
            .ok()
            .and_then(|s| s.parse::<bool>().ok())
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub enum ProverSystemConfig {
    #[cfg(feature = "sp1")]
    Succinct(crate::program_sp1::SP1ProverConfig),
    #[cfg(feature = "risc0")]
    RiscZero(crate::program_risc0::RiscZeroProverConfig),
    #[cfg(feature = "pico")]
    Pico(crate::program_pico::PicoProverConfig),
}

/// AWS Nitro Enclave attestation prover using zero-knowledge proofs.
///
/// `NitroEnclaveProver` is the main entry point for generating cryptographic proofs
/// of AWS Nitro Enclave attestation reports. It supports both single attestation
/// verification and batch processing with proof aggregation for improved efficiency.
///
/// The prover supports multiple zero-knowledge proof systems:
/// - **RISC0**: Industrial-grade zkVM with Bonsai cloud proving
/// - **SP1**: High-performance Succinct zkVM with network proving
///
/// # Architecture
///
/// The prover consists of two main ZK programs:
/// - **Verifier Program**: Validates individual attestation reports
/// - **Aggregator Program**: Combines multiple proofs into a single compact proof
///
/// # Examples
///
/// ## Basic Single Attestation Proof
///
/// ```rust,no_run
/// use aws_nitro_enclave_attestation_prover::{
///     NitroEnclaveProver, set_prover_dev_mode, ProverConfig
/// };
///
/// fn main() -> anyhow::Result<()> {
///     // Configure the prover (RISC0 example)
///     let config = ProverConfig::risc0();
///     
///     // Create prover instance
///     let prover = NitroEnclaveProver::new(config, None);
///     
///     // Load attestation report
///     let report_bytes = std::fs::read("samples/attestation_1.report")?;
///     
///     // Generate proof
///     let result = prover.prove_attestation_report(report_bytes)?;
///     
///     // Save proof result
///     std::fs::write("proof.json", result.encode_json()?)?;
///     
///     println!("Proof generated successfully!");
///     println!("{}", String::from_utf8_lossy(&result.encode_json()?));
///     // Submit the proof
///     
///     Ok(())
/// }
/// ```
///
/// ## Batch Proving with Aggregation
///
/// ```rust,no_run
/// use aws_nitro_enclave_attestation_prover::{
///     NitroEnclaveProver, set_prover_dev_mode, ProverConfig
/// };
///
/// fn prove_multiple_reports() -> anyhow::Result<()> {
///     let config = ProverConfig::sp1();
///     let prover = NitroEnclaveProver::new(config, None);
///     
///     // Load multiple attestation reports
///     let reports = vec![
///         std::fs::read("samples/attestation_1.report")?,
///         std::fs::read("samples/attestation_2.report")?,
///     ];
///     
///     // Generate aggregated proof for all reports
///     let reports_count = reports.len();
///     let result = prover.prove_multiple_reports(reports)?;
///     
///     println!("Aggregated proof generated for {} reports", reports_count);
///     println!("{}", String::from_utf8_lossy(&result.encode_json()?));
///     
///     // Submit the proof
///     
///     Ok(())
/// }
/// ```
///
/// ## Smart Contract Integration
///
/// For optimal gas efficiency, integrate with the Nitro Enclave Verifier contract:
///
/// ```rust,no_run
/// use aws_nitro_enclave_attestation_prover::{
///     NitroEnclaveProver, ProverConfig,
///     NitroEnclaveVerifierContract
/// };
/// use alloy_primitives::Address;
///
/// async fn prove_with_contract() -> anyhow::Result<()> {
///     // Connect to deployed verifier contract
///     let contract_address: Address = "0x1234567890123456789012345678901234567890".parse()?;
///     let rpc_url = "https://1rpc.io/holesky";
///     let verifier = NitroEnclaveVerifierContract::dial(rpc_url, contract_address, None)?;

///     let prover = NitroEnclaveProver::new(ProverConfig::sp1(), Some(verifier));
///     
///     let report_bytes = std::fs::read("samples/attestation_2.report")?;
///     
///     // Prove with contract optimization
///     let result = prover.prove_attestation_report(report_bytes)?;
///     
///     // The result.onchain_proof is ready for contract submission
///     std::fs::write("proof.json", result.encode_json()?)?;
///     
///     println!("Aggregation Proof generated successfully!");
///     println!("{}", String::from_utf8_lossy(&result.encode_json()?));
///     let result = prover.verify_on_chain(&result)?;
///     println!("onchain verfication result: {:?}", result);
///     
///     Ok(())
/// }
/// ```
///
pub struct NitroEnclaveProver {
    cfg: ProverConfig,
    /// Optional smart contract for optimized certificate verification
    contract: Option<NitroEnclaveVerifierContract>,
    /// Configuration for remote proving services
    remote_prover_config: Result<RemoteProverConfig, String>,
    /// ZK program for verifying individual attestation reports
    pub verifier: Box<dyn Program<Input = VerifierInput, Output = VerifierJournal>>,
    /// ZK program for aggregating multiple proofs into a single proof
    pub aggregator: Box<dyn Program<Input = BatchVerifierInput, Output = BatchVerifierJournal>>,
}

impl NitroEnclaveProver {
    /// Creates a new `NitroEnclaveProver` instance with the specified configuration.
    ///
    /// This constructor initializes the prover with the appropriate ZK programs
    /// (verifier and aggregator) based on the chosen proof system configuration.
    /// It also sets up environment variables for remote proving services if configured.
    ///
    /// # Arguments
    ///
    /// * `cfg` - The prover configuration specifying which ZK system to use (RISC0 or SP1)
    /// * `contract` - Optional smart contract for optimized certificate verification
    ///
    /// # Returns
    ///
    /// A new `NitroEnclaveProver` instance ready for proof generation
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use aws_nitro_enclave_attestation_prover::{NitroEnclaveProver, ProverConfig};
    ///
    /// // Create with RISC0 backend
    /// let config = ProverConfig::risc0();
    /// let prover = NitroEnclaveProver::new(config, None);
    /// ```
    pub fn new(cfg: ProverConfig, contract: Option<NitroEnclaveVerifierContract>) -> Self {
        match &cfg.system {
            #[cfg(feature = "sp1")]
            ProverSystemConfig::Succinct(system_cfg) => {
                use crate::program_sp1::{SP1_PROGRAM_AGGREGATOR, SP1_PROGRAM_VERIFIER};
                if let Some(api_url) = &system_cfg.rpc_url {
                    std::env::set_var("NETWORK_RPC_URL", api_url);
                }
                if let Some(api_key) = &system_cfg.private_key {
                    std::env::set_var("NETWORK_API_KEY", api_key);
                }
                NitroEnclaveProver {
                    contract,
                    remote_prover_config: system_cfg
                        .clone()
                        .try_into()
                        .map_err(|err| format!("{:?}", err)),
                    cfg,
                    verifier: Box::new(SP1_PROGRAM_VERIFIER.clone()),
                    aggregator: Box::new(SP1_PROGRAM_AGGREGATOR.clone()),
                }
            }
            #[cfg(feature = "risc0")]
            ProverSystemConfig::RiscZero(system_cfg) => {
                use crate::program_risc0::{RISC0_PROGRAM_AGGREGATOR, RISC0_PROGRAM_VERIFIER};

                // Set Boundless environment variables
                if let Some(rpc_url) = &system_cfg.rpc_url {
                    std::env::set_var("BOUNDLESS_RPC_URL", rpc_url);
                }
                if let Some(private_key) = &system_cfg.private_key {
                    std::env::set_var("BOUNDLESS_PRIVATE_KEY", private_key);
                }
                if let Some(program_url) = &system_cfg.program_url {
                    std::env::set_var("BOUNDLESS_PROGRAM_URL", program_url);
                }
                if let Some(min_price) = system_cfg.min_price {
                    std::env::set_var("BOUNDLESS_MIN_PRICE", min_price.to_string());
                }
                if let Some(max_price) = system_cfg.max_price {
                    std::env::set_var("BOUNDLESS_MAX_PRICE", max_price.to_string());
                }
                if let Some(timeout) = system_cfg.timeout {
                    std::env::set_var("BOUNDLESS_TIMEOUT", timeout.to_string());
                }
                if let Some(ramp_up_period) = system_cfg.ramp_up_period {
                    std::env::set_var("BOUNDLESS_RAMP_UP_PERIOD", ramp_up_period.to_string());
                }

                NitroEnclaveProver {
                    contract,
                    // RISC0 uses storage_provider_from_env() for uploads, not RemoteProverConfig
                    remote_prover_config: Ok(RemoteProverConfig {
                        api_url: system_cfg.rpc_url.clone(),
                        api_key: system_cfg.private_key.clone().unwrap_or_default(),
                    }),
                    cfg,
                    verifier: Box::new(RISC0_PROGRAM_VERIFIER.clone()),
                    aggregator: Box::new(RISC0_PROGRAM_AGGREGATOR.clone()),
                }
            }
            #[cfg(feature = "pico")]
            ProverSystemConfig::Pico(_) => {
                use crate::program_pico::{PICO_PROGRAM_AGGREGATOR, PICO_PROGRAM_VERIFIER};
                NitroEnclaveProver {
                    contract,
                    remote_prover_config: Err("Remote prover is not supported for Pico".to_string()),
                    cfg,
                    verifier: Box::new(PICO_PROGRAM_VERIFIER.clone()),
                    aggregator: Box::new(PICO_PROGRAM_AGGREGATOR.clone()),
                }
            }
        }
    }

    /// Returns the zero-knowledge coprocessor type used by this prover.
    ///
    /// This method identifies which ZK proof system (RISC0 or SP1) the prover
    /// instance is configured to use. Both verifier and aggregator are same zktype.
    ///
    /// # Returns
    ///
    /// The ZK coprocessor type enumeration value
    pub fn get_zk_type(&self) -> ZkCoProcessorType {
        self.verifier.zktype()
    }

    /// Returns the program identifiers for both verifier and aggregator circuits.
    ///
    /// These identifiers are used by smart contracts and verifiers to ensure
    /// they are validating proofs from the correct ZK programs.
    ///
    /// # Returns
    ///
    /// A `ProgramId` struct containing:
    /// - `verifier_id`: Hash of the verifier program
    /// - `verifier_proof_id`: Hash of the aggregator to verify the verifier's proof
    /// - `aggregator_id`: Hash of the aggregator program
    pub fn get_program_id(&self) -> ProgramId {
        ProgramId {
            verifier_id: self.verifier.program_id(),
            verifier_proof_id: self.verifier.verify_proof_id(),
            aggregator_id: self.aggregator.program_id(),
        }
    }

    /// Converts a raw ZK proof into a format suitable for onchain verification.
    ///
    /// This method transforms the internal proof representation into bytes
    /// that can be submitted to smart contracts for on-chain verification.
    /// The exact encoding depends on the underlying ZK system (RISC0 or SP1).
    ///
    /// # Arguments
    ///
    /// * `proof` - The raw proof to be encoded
    ///
    pub fn encode_proof_for_onchain(&self, proof: &RawProof) -> anyhow::Result<Bytes> {
        self.verifier.onchain_proof(proof)
    }

    /// Uploads both verifier and aggregator program images to the remote proving service.
    ///
    /// This method deploys the ZK programs to remote infrastructure (like Bonsai for RISC0
    /// or SP1 Network for SP1) to enable faster cloud-based proof generation.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use aws_nitro_enclave_attestation_prover::{NitroEnclaveProver, ProverConfig};
    ///
    /// fn main() -> anyhow::Result<()> {
    ///     let prover = NitroEnclaveProver::new(ProverConfig::risc0(), None);
    ///     let program_id = prover.upload_program_images()?;
    ///     println!("Programs uploaded successfully: {:?}", program_id);
    ///     Ok(())
    /// }
    /// ```
    pub fn upload_program_images(&self) -> anyhow::Result<ProgramId> {
        let cfg = match &self.remote_prover_config {
            Ok(cfg) => cfg,
            Err(err) => return Err(anyhow!("{}", err)),
        };
        self.verifier.upload_image(&cfg)?;
        self.aggregator.upload_image(&cfg)?;
        Ok(self.get_program_id())
    }

    /// Generates multiple composite proofs concurrently from verifier inputs.
    ///
    /// This method processes multiple attestation reports in parallel to generate
    /// individual proofs that can later be aggregated. It respects the concurrency
    /// limit set by the `PROVE_MAX_CONCURRENCY` environment variable (default: 8).
    ///
    /// # Arguments
    ///
    /// * `inputs` - Array of prepared verifier inputs for proof generation
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<RawProof>)` - Vector of generated proofs, one per input
    /// * `Err(anyhow::Error)` - If any proof generation fails
    ///
    /// # Performance
    ///
    /// The concurrency level can be controlled via the `PROVE_MAX_CONCURRENCY`
    /// environment variable to balance performance with system resources.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use aws_nitro_enclave_attestation_prover::{NitroEnclaveProver, ProverConfig};
    ///
    /// fn main() -> anyhow::Result<()> {
    ///     let prover = NitroEnclaveProver::new(ProverConfig::risc0(), None);
    ///     let reports = vec![std::fs::read("samples/attestation_1.report")?];
    ///     let inputs = prover.prepare_verifier_inputs(reports)?;
    ///     let proofs = prover.gen_multi_composite_proofs(&inputs)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn gen_multi_composite_proofs(
        &self,
        inputs: &[VerifierInput],
    ) -> anyhow::Result<Vec<RawProof>> {
        let max_concurrency = std::env::var("PROVE_MAX_CONCURRENCY")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or_else(|| 8);

        // Generate partial proofs in parallel
        Ok(parallels_blocking(max_concurrency, inputs, |input| {
            self.verifier
                .gen_proof(input, RawProofType::Composite, None)
        })?)
    }

    /// Aggregates multiple individual proofs into a single compact proof.
    ///
    /// This method combines multiple verification proofs into a single aggregated
    /// proof that proves all the individual attestations simultaneously. This is
    /// more gas-efficient for on-chain verification when dealing with multiple reports.
    ///
    /// # Arguments
    ///
    /// * `proofs` - Vector of individual proofs to be aggregated
    ///
    /// # Gas Efficiency
    ///
    /// Aggregated proofs provide significant gas savings on-chain compared to
    /// verifying multiple individual proofs separately.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use aws_nitro_enclave_attestation_prover::{NitroEnclaveProver, ProverConfig};
    ///
    /// fn main() -> anyhow::Result<()> {
    ///     let prover = NitroEnclaveProver::new(ProverConfig::risc0(), None);
    ///     let reports = vec![std::fs::read("samples/attestation_1.report")?];
    ///     let inputs = prover.prepare_verifier_inputs(reports)?;
    ///     let individual_proofs = prover.gen_multi_composite_proofs(&inputs)?;
    ///     let aggregated_proof = prover.aggregate_proofs(individual_proofs)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn aggregate_proofs(&self, proofs: Vec<RawProof>) -> anyhow::Result<RawProof> {
        let mut journals = Vec::with_capacity(proofs.len());
        let mut encoded_proofs = Vec::with_capacity(proofs.len());
        for item in &proofs {
            let decoded = item.decode_journal::<VerifierJournal>()?;
            journals.push(decoded);
            encoded_proofs.push(&item.encoded_proof);
        }

        let batch_input = BatchVerifierInput {
            verifierVk: self.verifier.verify_proof_id(),
            outputs: journals,
        };
        Ok(self.aggregator.gen_proof(
            &batch_input,
            RawProofType::Groth16,
            Some(encoded_proofs.as_slice()),
        )?)
    }

    /// Generates a zero-knowledge proof for a single AWS Nitro Enclave attestation report.
    ///
    /// This is the primary method for proving individual attestation reports. It handles
    /// the complete workflow from parsing the raw report to generating a blockchain-ready proof.
    ///
    /// # Arguments
    ///
    /// * `report_bytes` - Raw attestation report bytes from AWS Nitro Enclave
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use aws_nitro_enclave_attestation_prover::{NitroEnclaveProver, ProverConfig};
    ///
    /// fn main() -> anyhow::Result<()> {
    ///     let prover = NitroEnclaveProver::new(ProverConfig::risc0(), None);
    ///     let report_bytes = std::fs::read("samples/attestation_1.report")?;
    ///     let proof = prover.prove_attestation_report(report_bytes)?;
    ///
    ///     // Submit to blockchain or save for later use
    ///     std::fs::write("proof.json", proof.encode_json()?)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn prove_attestation_report(&self, report_bytes: Vec<u8>) -> anyhow::Result<OnchainProof> {
        let inputs = self.prepare_verifier_inputs(vec![report_bytes])?;
        let proof = self
            .verifier
            .gen_proof(&inputs[0], RawProofType::Groth16, None)?;
        Ok(self.create_onchain_proof(proof, ProofType::Verifier)?)
    }

    /// Generates an aggregated zero-knowledge proof for multiple attestation reports.
    ///
    /// This method is optimized for batch processing multiple attestation reports
    /// simultaneously. It generates individual proofs in parallel and then aggregates
    /// them into a single compact proof for efficient on-chain verification.
    ///
    /// # Arguments
    ///
    /// * `raw_reports` - Vector of raw attestation report bytes
    ///
    /// # Performance
    ///
    /// This method provides significant performance benefits for multiple reports:
    /// - Parallel proof generation reduces total proving time
    /// - Aggregated proofs reduce on-chain verification costs
    /// - Amortized certificate validation overhead
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use aws_nitro_enclave_attestation_prover::{NitroEnclaveProver, ProverConfig};
    ///
    /// fn main() -> anyhow::Result<()> {
    ///     let prover = NitroEnclaveProver::new(ProverConfig::sp1(), None);
    ///     let reports = vec![
    ///         std::fs::read("samples/attestation_1.report")?,
    ///         std::fs::read("samples/attestation_2.report")?,
    ///     ];
    ///
    ///     let aggregated_proof = prover.prove_multiple_reports(reports)?;
    ///     println!("Generated aggregated proof: {:?}", aggregated_proof);
    ///     Ok(())
    /// }
    /// ```
    pub fn prove_multiple_reports(
        &self,
        raw_reports: Vec<Vec<u8>>,
    ) -> anyhow::Result<OnchainProof> {
        let inputs = self.prepare_verifier_inputs(raw_reports)?;
        let proofs = self.gen_multi_composite_proofs(&inputs)?;
        let result = self.aggregate_proofs(proofs)?;
        Ok(self.create_onchain_proof(result, ProofType::Aggregator)?)
    }

    /// Prepares verifier inputs from raw AWS Nitro Enclave attestation reports.
    ///
    /// This method performs the complete preprocessing pipeline for attestation reports:
    /// 1. Parses raw attestation reports and validates their structure
    /// 2. Extracts certificate chains and computes their cryptographic digests
    /// 3. Queries the smart contract (if available) for trusted certificate information
    /// 4. Constructs properly formatted inputs for the ZK verifier program
    ///
    /// # Arguments
    ///
    /// * `raw_reports` - Vector of raw attestation report bytes from AWS Nitro Enclaves
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<VerifierInput>)` - Vector of prepared inputs for ZK proof generation
    /// * `Err(anyhow::Error)` - If parsing, validation, or contract interaction fails
    ///
    /// # Contract Integration
    ///
    /// When a smart contract is configured, this method:
    /// - Queries the contract's certificate cache for trusted certificate lengths
    /// - Optimizes proof generation by using pre-verified certificate chains
    /// - Reduces gas costs for on-chain verification
    ///
    /// When no contract is provided:
    /// - Issues warnings about potential verification failures and increased costs
    /// - Validates report timestamps against current time
    /// - Defaults to trusting only the root certificate (length = 1)
    ///
    /// # Security Considerations
    ///
    /// - Reports older than 3 hour trigger warnings as they may indicate stale attestations
    /// - Certificate chain validation is critical for ensuring attestation authenticity
    /// - Smart contract integration is recommended for production environments
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use aws_nitro_enclave_attestation_prover::{NitroEnclaveProver, ProverConfig};
    ///
    /// fn main() -> anyhow::Result<()> {
    ///     let prover = NitroEnclaveProver::new(ProverConfig::risc0(), None);
    ///     let reports = vec![
    ///         std::fs::read("attestation1.report")?,
    ///         std::fs::read("attestation2.report")?,
    ///     ];
    ///
    ///     let inputs = prover.prepare_verifier_inputs(reports)?;
    ///     println!("Prepared {} inputs for verification", inputs.len());
    ///     Ok(())
    /// }
    /// ```
    pub fn prepare_verifier_inputs(
        &self,
        raw_reports: Vec<Vec<u8>>,
    ) -> anyhow::Result<Vec<VerifierInput>> {
        let mut parsed_reports = Vec::with_capacity(raw_reports.len());
        let mut cert_digests = Vec::with_capacity(raw_reports.len());

        // Parse attestation reports and extract certificate chain digests
        for raw_report in &raw_reports {
            parsed_reports.push(AttestationReport::parse(&raw_report)?);
            let cert_chain = parsed_reports.last().unwrap().cert_chain()?;
            cert_digests.push(cert_chain.digest().to_vec());
        }

        let trusted_certs_prefix_lengths;
        let max_time_diff;
        match &self.contract {
            Some(verifier_contract) => {
                // make sure the zk config aligned
                let zk_config = block_on(verifier_contract.zk_config(self.verifier.zktype()))?;
                max_time_diff = block_on(verifier_contract.max_time_diff())?;

                let program_id = self.get_program_id();
                let verify_result = program_id.verify(&zk_config).with_context(|| {
                    format!("Failed to verify zkconfig for {:?}", self.verifier.zktype())
                });
                if let Err(verify_err) = verify_result {
                    if !self.cfg.skip_contract_program_id_check {
                        bail!(
                            "Program ID verification failed: {:?}. Set SKIP_CONTRACT_PROGRAM_ID_CHECK=true to ignore this check.",
                            verify_err
                        );
                    } else {
                        tracing::warn!("Program ID verification failed: {:?}.", verify_err);
                    }
                }

                // Query smart contract for certificate cache information
                trusted_certs_prefix_lengths =
                    block_on(verifier_contract.batch_query_cert_cache(cert_digests))?;
            }
            None => {
                tracing::warn!("Contract not provided, may lead to attestation failures and increased costs. Not recommended for production.");
                max_time_diff = 3600 * 3;
                trusted_certs_prefix_lengths =
                    vec![self.cfg.default_trusted_certs_prefix_length; parsed_reports.len()];
            }
        }

        // Validate report timestamps when no contract is available
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        for (idx, report) in parsed_reports.iter().enumerate() {
            let report_timestamp = report.doc().timestamp / 1000;
            if report_timestamp + max_time_diff < current_time {
                if self.cfg.skip_time_validity_check {
                    tracing::warn!(
                        "Report[{idx}] signed {} seconds ago, ignoring time validity check.",
                        current_time - report_timestamp
                    );
                } else {
                    bail!(
                        "Report[{idx}] signed {} seconds ago, may indicate verification failure. set `SKIP_TIME_VALIDITY_CHECK=true` to ignore this check.",
                        current_time - report_timestamp
                    );
                }
            }
        }

        assert!(
            trusted_certs_prefix_lengths.len() == raw_reports.len(),
            "Trusted certificate lengths count mismatch"
        );

        // Build verifier inputs with trusted certificate information
        let verifier_inputs = raw_reports
            .into_iter()
            .zip(trusted_certs_prefix_lengths)
            .map(|(report_bytes, trusted_cert_prefix_len)| VerifierInput {
                trustedCertsPrefixLen: trusted_cert_prefix_len,
                attestationReport: report_bytes.into(),
            })
            .collect();
        Ok(verifier_inputs)
    }

    /// Builds a complete proof result with all metadata for blockchain submission.
    ///
    /// This method constructs an `OnchainProof` structure that packages the raw ZK proof
    /// along with all necessary metadata required for on-chain verification. The resulting
    /// proof package is ready for submission to smart contracts.
    ///
    /// # Arguments
    ///
    /// * `raw_proof` - The raw zero-knowledge proof generated by the ZK program
    /// * `proof_type` - The type of proof (Verifier for single attestations, Aggregator for batch)
    ///
    /// # Proof Package Contents
    ///
    /// The resulting `OnchainProof` contains:
    /// - Proof bytes suitable for smart contract verification
    /// - Program identifiers for verification logic validation
    /// - Proof type metadata for correct contract method selection
    /// - ZK system information (RISC0 or SP1)
    /// - Serialization helpers for JSON export
    ///
    /// # Usage
    ///
    /// This method is typically called internally by `prove_attestation_report()`
    /// and `prove_multiple_reports()`. Direct usage is for advanced scenarios
    /// where custom proof processing is required.
    /// ```
    pub fn create_onchain_proof(
        &self,
        raw_proof: RawProof,
        proof_type: ProofType,
    ) -> anyhow::Result<OnchainProof> {
        Ok(OnchainProof::new_from_program(
            &*self.verifier,
            self.get_program_id(),
            raw_proof,
            proof_type,
        )?)
    }

    /// Verifies a zero-knowledge proof on the Ethereum blockchain via smart contract.
    ///
    /// This method submits a previously generated ZK proof to the deployed Nitro Enclave
    /// Verifier smart contract for on-chain verification.
    ///
    /// # Arguments
    ///
    /// * `proof` - The result generated by `prove_attestation_report()` or `prove_multiple_reports()`
    ///
    /// # Prerequisites
    ///
    /// This method requires that the prover was initialized with a valid smart contract:
    /// - The contract must be deployed and accessible via the configured RPC endpoint
    /// - The contract must support the proof type being verified (RISC0 or SP1)
    /// - The program identifiers in the proof must match those registered in the contract
    ///
    pub fn verify_on_chain(
        &self,
        proof: &OnchainProof,
    ) -> anyhow::Result<OnchainProofVerifyResult> {
        let contract = self
            .contract
            .as_ref()
            .ok_or_else(|| anyhow!("verify on chain requires contract info"))?;
        let result = block_on(contract.verify_proof(proof))
            .map_err(|err| anyhow!("Failed to verify proof on chain: {}", err))?;
        Ok(result)
    }
}
