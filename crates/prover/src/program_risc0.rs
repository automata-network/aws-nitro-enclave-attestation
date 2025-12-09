use std::marker::PhantomData;
use std::time::Duration;

use alloy_primitives::{hex, Bytes, B256};
use alloy_sol_types::SolValue;
use anyhow::{anyhow, Context};
use aws_nitro_enclave_attestation_verifier::stub::{
    BatchVerifierInput, BatchVerifierJournal, VerifierInput, VerifierJournal, ZkCoProcessorType,
};
use boundless_market::{
    alloy::{
        primitives::{utils::parse_units, U256},
        providers::{Provider, ProviderBuilder},
        signers::local::PrivateKeySigner,
        transports::http::reqwest::Url,
    },
    client::Client as BoundlessClient,
    contracts::FulfillmentData,
    request_builder::OfferParams,
    storage::storage_provider_from_env,
    Deployment,
    StorageProvider,
};
use lazy_static::lazy_static;
use risc0_ethereum_contracts::{groth16, receipt::decode_seal_with_claim};
use risc0_methods::{
    RISC0_AGGREGATOR_ELF, RISC0_AGGREGATOR_ID, RISC0_VERIFIER_ELF, RISC0_VERIFIER_ID,
};
use risc0_zkvm::{
    default_executor, Digest, ExecutorEnv, InnerReceipt, Receipt, ReceiptClaim, VERSION,
    compute_image_id,
};
use serde::Serialize;

use crate::{
    program::{Program, RemoteProverConfig},
    utils::block_on,
    RawProof, RawProofType,
};

lazy_static! {
    pub static ref RISC0_PROGRAM_VERIFIER: ProgramRisc0<VerifierInput, VerifierJournal> =
        ProgramRisc0::new(RISC0_VERIFIER_ELF, *RISC0_VERIFIER_ID);
    pub static ref RISC0_PROGRAM_AGGREGATOR: ProgramRisc0<BatchVerifierInput, BatchVerifierJournal> =
        ProgramRisc0::new(RISC0_AGGREGATOR_ELF, *RISC0_AGGREGATOR_ID);
}

/// Proof type for Boundless network proving
#[derive(Debug, Clone, Copy, Default)]
pub enum BoundlessProofType {
    /// Groth16 proof - on-chain verifiable
    #[default]
    Groth16,
    /// Merkle proof
    Merkle,
}

#[derive(Debug, Clone)]
pub struct RiscZeroProverConfig {
    /// Boundless RPC URL (env: BOUNDLESS_RPC_URL)
    pub rpc_url: Option<String>,
    /// Wallet private key hex (env: BOUNDLESS_PRIVATE_KEY)
    pub private_key: Option<String>,
    /// Optional verifier program URL for pre-uploaded ELF (env: BOUNDLESS_VERIFIER_PROGRAM_URL)
    pub verifier_program_url: Option<String>,
    /// Optional aggregator program URL for pre-uploaded ELF (env: BOUNDLESS_AGGREGATOR_PROGRAM_URL)
    pub aggregator_program_url: Option<String>,
    /// Proof type: Groth16 or Merkle (default: Groth16)
    pub proof_type: BoundlessProofType,
    /// Minimum price in wei per cycle
    pub min_price: Option<u128>,
    /// Maximum price in wei per cycle
    pub max_price: Option<u128>,
    /// Timeout in seconds
    pub timeout: Option<u32>,
    /// Ramp-up period in seconds
    pub ramp_up_period: Option<u32>,
}

impl Default for RiscZeroProverConfig {
    fn default() -> Self {
        RiscZeroProverConfig {
            rpc_url: std::env::var("BOUNDLESS_RPC_URL").ok(),
            private_key: std::env::var("BOUNDLESS_PRIVATE_KEY").ok(),
            verifier_program_url: std::env::var("BOUNDLESS_VERIFIER_PROGRAM_URL").ok(),
            aggregator_program_url: std::env::var("BOUNDLESS_AGGREGATOR_PROGRAM_URL").ok(),
            proof_type: BoundlessProofType::default(),
            min_price: std::env::var("BOUNDLESS_MIN_PRICE")
                .ok()
                .and_then(|s| s.parse().ok()),
            max_price: std::env::var("BOUNDLESS_MAX_PRICE")
                .ok()
                .and_then(|s| s.parse().ok()),
            timeout: std::env::var("BOUNDLESS_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok()),
            ramp_up_period: std::env::var("BOUNDLESS_RAMP_UP_PERIOD")
                .ok()
                .and_then(|s| s.parse().ok()),
        }
    }
}

impl RiscZeroProverConfig {
    // Default constants for Boundless pricing and timing
    const DEFAULT_MIN_PRICE_GWEI: &'static str = "0.0001";
    const DEFAULT_MAX_PRICE_GWEI: &'static str = "0.001";
    const DEFAULT_COLLATERAL_ETH: &'static str = "10";
    const DEFAULT_TIMEOUT_BUFFER: u32 = 600;
    const DEFAULT_POLL_INTERVAL_SECS: u64 = 5;

    /// Get effective min price (config or default: 0.0001 gwei)
    pub fn effective_min_price(&self) -> U256 {
        self.min_price
            .map(U256::from)
            .unwrap_or_else(|| parse_units(Self::DEFAULT_MIN_PRICE_GWEI, "gwei").unwrap().into())
    }

    /// Get effective max price (config or default: 0.001 gwei)
    pub fn effective_max_price(&self) -> U256 {
        self.max_price
            .map(U256::from)
            .unwrap_or_else(|| parse_units(Self::DEFAULT_MAX_PRICE_GWEI, "gwei").unwrap().into())
    }

    /// Get effective collateral (default: 10 ETH)
    pub fn effective_collateral(&self) -> U256 {
        parse_units(Self::DEFAULT_COLLATERAL_ETH, "ether").unwrap().into()
    }

    /// Get effective timeout with buffer
    pub fn effective_timeout(&self) -> Option<(u32, u32)> {
        self.timeout.map(|t| (t, t + Self::DEFAULT_TIMEOUT_BUFFER))
    }

    /// Get poll interval for waiting on fulfillment
    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(Self::DEFAULT_POLL_INTERVAL_SECS)
    }
}

/// Input format for the aggregator when used with Boundless.
/// Must match the struct in risc0-aggregator guest program.
#[derive(Serialize)]
struct AggregatorInput {
    input_bytes: Vec<u8>,
    receipts: Vec<Receipt>,
}

#[derive(Clone)]
pub struct ProgramRisc0<Input, Output> {
    elf: &'static [u8],
    image_id: [u32; 8],
    _marker: PhantomData<(Input, Output)>,
}

impl<Input, Output> ProgramRisc0<Input, Output> {
    pub fn new(elf: &'static [u8], image_id: [u32; 8]) -> Self {
        ProgramRisc0 {
            elf,
            image_id,
            _marker: PhantomData,
        }
    }

    /// Submit a proof request to Boundless and wait for fulfillment.
    /// Returns (seal_bytes, journal_bytes) on success.
    /// If `program_url` is provided, uses that instead of uploading the ELF.
    async fn submit_boundless_request(
        stdin: &[u8],
        elf: &[u8],
        cfg: &RiscZeroProverConfig,
        program_url: Option<&str>,
        force_groth16: bool,
    ) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
        let rpc_url = cfg
            .rpc_url
            .as_ref()
            .ok_or_else(|| anyhow!("missing BOUNDLESS_RPC_URL"))?;
        let private_key_hex = cfg
            .private_key
            .as_ref()
            .ok_or_else(|| anyhow!("missing BOUNDLESS_PRIVATE_KEY"))?;

        let rpc_url_parsed: Url = rpc_url
            .parse()
            .context("Failed to parse Boundless RPC URL")?;

        let provider = ProviderBuilder::new().connect_http(rpc_url_parsed.clone());
        let chain_id = provider
            .get_chain_id()
            .await
            .context("Failed to get chain ID from RPC")?;

        let deployment = Deployment::from_chain_id(chain_id)
            .with_context(|| format!("No Boundless deployment for chain {}", chain_id))?;

        let private_key_bytes = hex::decode(private_key_hex.trim_start_matches("0x"))
            .context("Failed to decode private key (must be hex-encoded)")?;
        let private_key = PrivateKeySigner::from_slice(&private_key_bytes)
            .context("Failed to parse private key")?;

        let storage_provider = storage_provider_from_env()
            .context("Failed to get storage provider (check PINATA_JWT env var)")?;

        let client = BoundlessClient::builder()
            .with_rpc_url(rpc_url_parsed)
            .with_deployment(deployment)
            .with_storage_provider(Some(storage_provider))
            .with_private_key(private_key)
            .config_offer_layer(|config| {
                config
                    .max_price_per_cycle(cfg.effective_max_price())
                    .min_price_per_cycle(cfg.effective_min_price())
            })
            .build()
            .await
            .context("Failed to build Boundless client")?;

        // Build request
        let mut request_builder = client.new_request().with_stdin(stdin);

        // Set program (URL if provided, otherwise upload ELF)
        if let Some(url) = program_url {
            request_builder = request_builder
                .with_program_url(url)
                .context("Failed to set program URL")?;
        } else {
            request_builder = request_builder.with_program(elf.to_vec());
        }

        // Set proof type
        if force_groth16 {
            request_builder = request_builder.with_groth16_proof();
        } else {
            match cfg.proof_type {
                BoundlessProofType::Groth16 => {
                    request_builder = request_builder.with_groth16_proof();
                }
                BoundlessProofType::Merkle => {
                    // Merkle is the default, no flag needed
                }
            }
        }

        // Configure offer params
        let mut offer_builder = OfferParams::builder();
        if let Some(min_price) = cfg.min_price {
            offer_builder.min_price(U256::from(min_price));
        }
        if let Some(max_price) = cfg.max_price {
            offer_builder.max_price(U256::from(max_price));
        }
        if let Some((lock_timeout, timeout)) = cfg.effective_timeout() {
            offer_builder.lock_timeout(lock_timeout);
            offer_builder.timeout(timeout + 600);
        }
        if let Some(ramp_up_period) = cfg.ramp_up_period {
            offer_builder.ramp_up_period(ramp_up_period);
        }
        offer_builder.lock_collateral(cfg.effective_collateral());
        request_builder = request_builder.with_offer(offer_builder);

        tracing::debug!("Boundless request: {:?}", &request_builder);

        // Submit and wait for fulfillment
        let (request_id, expires_at) = client
            .submit_onchain(request_builder)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to submit proof request: {:?}", e))?;

        tracing::info!("Boundless request submitted: {:x}", request_id);

        let fulfillment = client
            .wait_for_request_fulfillment(request_id, cfg.poll_interval(), expires_at)
            .await
            .context("Failed waiting for proof fulfillment")?;

        // Decode the fulfillment data to extract the journal
        // fulfillmentData is ABI-encoded FulfillmentDataImageIdAndJournal, not raw journal bytes
        let fulfillment_data = fulfillment
            .data()
            .context("Failed to decode fulfillment data")?;
        let journal = match fulfillment_data {
            FulfillmentData::ImageIdAndJournal(_, journal) => journal.to_vec(),
            FulfillmentData::None => {
                return Err(anyhow!("Fulfillment has no journal data"));
            }
            _ => {
                return Err(anyhow!("Unexpected fulfillment data type"));
            }
        };

        Ok((fulfillment.seal.to_vec(), journal))
    }

    /// Dev mode: execute zkVM without proof generation
    fn gen_dev_proof(&self, input_bytes: &[u8]) -> anyhow::Result<RawProof> {
        let env = ExecutorEnv::builder()
            .write_slice(input_bytes)
            .build()?;

        let executor = default_executor();
        let session = executor.execute(env, self.elf)?;

        // Return mock proof with real journal from execution
        let journal: Bytes = session.journal.bytes.clone().into();
        Ok(RawProof {
            encoded_proof: Bytes::new(), // Empty proof in dev mode
            journal,
        })
    }

    /// Dev mode: execute aggregator zkVM without proof generation.
    /// Deserializes full Receipts from bincode-encoded data.
    fn gen_dev_aggregated_proof(
        input_bytes: &[u8],
        encoded_proofs: &[&Bytes],
    ) -> anyhow::Result<RawProof> {
        // Decode BatchVerifierInput to verify array length
        let batch_input = BatchVerifierInput::decode(input_bytes)
            .context("Failed to decode BatchVerifierInput")?;

        // Verify array lengths match
        if encoded_proofs.len() != batch_input.outputs.len() {
            return Err(anyhow!(
                "Number of proofs ({}) must match number of outputs ({})",
                encoded_proofs.len(),
                batch_input.outputs.len()
            ));
        }

        // Deserialize full Receipts (which include the correct journal bytes)
        let receipts: Vec<Receipt> = encoded_proofs
            .iter()
            .map(|encoded_receipt| {
                bincode::deserialize(encoded_receipt)
                    .context("Failed to deserialize Receipt")
            })
            .collect::<anyhow::Result<_>>()?;

        // Build AggregatorInput (postcard + length-prefix framing)
        let aggregator_input = AggregatorInput {
            input_bytes: input_bytes.to_vec(),
            receipts,
        };

        let input_frame = postcard::to_allocvec(&aggregator_input)
            .context("Failed to serialize aggregator input with postcard")?;
        let mut framed_input = Vec::with_capacity(4 + input_frame.len());
        framed_input.extend_from_slice(&(input_frame.len() as u32).to_le_bytes());
        framed_input.extend_from_slice(&input_frame);

        // Execute aggregator in dev mode
        let env = ExecutorEnv::builder()
            .write_slice(&framed_input)
            .build()?;

        let executor = default_executor();
        let session = executor.execute(env, RISC0_AGGREGATOR_ELF)?;

        Ok(RawProof {
            encoded_proof: Bytes::new(), // Empty proof in dev mode
            journal: session.journal.bytes.clone().into(),
        })
    }

    /// Production: generate proof via Boundless network.
    ///
    /// Serializes full `Receipt` (including journal) so aggregation can reconstruct it exactly.
    fn gen_proof_boundless(
        &self,
        input_bytes: &[u8],
        _raw_proof_type: RawProofType,
        cfg: &RiscZeroProverConfig,
    ) -> anyhow::Result<RawProof> {
        let image_id = Digest::new(self.image_id);
        let program_url = cfg.verifier_program_url.as_deref();

        block_on(async {
            let (seal_bytes, journal_bytes) =
                Self::submit_boundless_request(input_bytes, self.elf, cfg, program_url, true).await?;

            // Construct Receipt from fulfillment - stores full Receipt for aggregation
            let receipt = Self::construct_receipt(seal_bytes, journal_bytes.clone(), image_id)?;

            // Store full Receipt (not just InnerReceipt) to preserve journal bytes exactly
            Ok(RawProof::from_proof(&receipt, journal_bytes.into())?)
        })
    }

    /// Production: generate aggregated proof via Boundless.
    /// Deserializes full Receipts from bincode-encoded data.
    fn gen_proof_boundless_aggregated(
        input_bytes: &[u8],
        encoded_proofs: &[&Bytes],
        cfg: &RiscZeroProverConfig,
    ) -> anyhow::Result<RawProof> {
        // Decode BatchVerifierInput to verify array length
        let batch_input = BatchVerifierInput::decode(input_bytes)
            .context("Failed to decode BatchVerifierInput")?;

        // Verify array lengths match
        if encoded_proofs.len() != batch_input.outputs.len() {
            return Err(anyhow!(
                "Number of proofs ({}) must match number of outputs ({})",
                encoded_proofs.len(),
                batch_input.outputs.len()
            ));
        }

        // Deserialize full Receipts (which include the correct journal bytes)
        let receipts: Vec<Receipt> = encoded_proofs
            .iter()
            .map(|encoded_receipt| {
                bincode::deserialize(encoded_receipt)
                    .context("Failed to deserialize Receipt")
            })
            .collect::<anyhow::Result<_>>()?;

        // Build AggregatorInput (postcard + length-prefix framing)
        let aggregator_input = AggregatorInput {
            input_bytes: input_bytes.to_vec(),
            receipts,
        };

        let input_frame = postcard::to_allocvec(&aggregator_input)
            .context("Failed to serialize aggregator input with postcard")?;
        let mut framed_input = Vec::with_capacity(4 + input_frame.len());
        framed_input.extend_from_slice(&(input_frame.len() as u32).to_le_bytes());
        framed_input.extend_from_slice(&input_frame);

        let program_url = cfg.aggregator_program_url.as_deref();

        block_on(async {
            let (seal_bytes, journal_bytes) =
                Self::submit_boundless_request(&framed_input, RISC0_AGGREGATOR_ELF, cfg, program_url, true).await?;

            // Construct Receipt from fulfillment (seal has selector prefix)
            let aggregator_image_id = compute_image_id(RISC0_AGGREGATOR_ELF)?;
            let receipt = Self::construct_receipt(seal_bytes, journal_bytes.clone(), aggregator_image_id)?;

            Ok(RawProof::from_proof(&receipt, journal_bytes.into())?)
        })
    }

    /// Construct a Receipt from Boundless fulfillment data.
    /// The seal should include the 4-byte selector prefix followed by ABI-encoded proof.
    fn construct_receipt(
        seal: Vec<u8>,
        journal: Vec<u8>,
        image_id: Digest,
    ) -> anyhow::Result<Receipt> {
        // Handle empty seals (dev mode) - create FakeReceipt
        if seal.is_empty() {
            let claim = ReceiptClaim::ok(image_id, journal.clone());
            return Ok(Receipt::new(
                InnerReceipt::Fake(risc0_zkvm::FakeReceipt::new(claim)),
                journal,
            ));
        }

        // Use decode_seal_with_claim which:
        // - Reads selector from first 4 bytes to determine seal type
        // - Extracts correct verifier parameters from the selector
        // - Handles Groth16, FakeReceipt, and SetVerifier seal types
        let claim = ReceiptClaim::ok(image_id, journal.clone());
        let receipt = decode_seal_with_claim(
            alloy_primitives::Bytes::from(seal),
            claim,
            journal,
        )
        .context("Failed to decode seal")?;

        receipt
            .receipt()
            .cloned()
            .ok_or_else(|| anyhow!("Expected base receipt, got set inclusion receipt"))
    }
}

impl<Input, Output> Program for ProgramRisc0<Input, Output>
where
    Input: SolValue + Send + Sync,
    Output: SolValue + Send + Sync,
{
    type Input = Input;
    type Output = Output;
    fn version(&self) -> &'static str {
        VERSION
    }
    fn zktype(&self) -> ZkCoProcessorType {
        ZkCoProcessorType::RiscZero
    }

    fn onchain_proof(&self, proof: &RawProof) -> anyhow::Result<Bytes> {
        let receipt = proof.decode_proof::<Receipt>()?;
        let encoded_proof = match &receipt.inner {
            InnerReceipt::Groth16(groth16_receipt) => groth16::encode(&groth16_receipt.seal)?,
            _ => vec![],
        };
        Ok(encoded_proof.into())
    }

    fn upload_image(&self, _cfg: &RemoteProverConfig) -> anyhow::Result<()> {
        block_on(async {
            let storage_provider = storage_provider_from_env()
                .context("Failed to get storage provider (check PINATA_JWT env var)")?;

            let elf_url = storage_provider
                .upload_input(self.elf)
                .await
                .context("Failed to upload ELF to Pinata/IPFS")?;

            tracing::info!(
                "Uploaded image {} to storage: {}",
                Digest::new(self.image_id),
                elf_url
            );

            Ok(())
        })
    }

    fn program_id(&self) -> B256 {
        B256::from_slice(Digest::new(self.image_id).as_bytes())
    }

    fn verify_proof_id(&self) -> B256 {
        self.program_id()
    }

    /// Generates a zero-knowledge proof for the given input.
    ///
    /// # RISC0/Boundless Specifics
    ///
    /// For RISC0 with Boundless, `encoded_composite_proofs` contains **bincode-serialized
    /// `risc0_zkvm::Receipt` objects** (not compressed proofs as in SP1/Pico). This is
    /// because Boundless aggregation requires full Receipt objects to be passed to the
    /// aggregator guest program.
    ///
    /// - `RawProofType::Composite`: Generates proof for later aggregation (serializes full Receipt)
    /// - `RawProofType::Groth16`: Generates final on-chain verifiable proof (serializes InnerReceipt)
    fn gen_proof(
        &self,
        input: &Self::Input,
        raw_proof_type: RawProofType,
        encoded_composite_proofs: Option<&[&Bytes]>,
    ) -> anyhow::Result<RawProof> {
        let dev_mode = std::env::var("RISC0_DEV_MODE")
            .map(|v| v == "1")
            .unwrap_or(false);

        let input_bytes = input.abi_encode();
        let is_aggregation = encoded_composite_proofs.is_some();

        if dev_mode {
            if is_aggregation {
                // Dev mode aggregation: execute aggregator with receipts
                Self::gen_dev_aggregated_proof(&input_bytes, encoded_composite_proofs.unwrap())
            } else {
                // Dev mode single proof
                self.gen_dev_proof(&input_bytes)
            }
        } else {
            let cfg = RiscZeroProverConfig::default();
            if is_aggregation {
                // Boundless aggregation: deserialize receipts and submit to aggregator
                Self::gen_proof_boundless_aggregated(
                    &input_bytes,
                    encoded_composite_proofs.unwrap(),
                    &cfg,
                )
            } else {
                // Boundless single proof
                self.gen_proof_boundless(&input_bytes, raw_proof_type, &cfg)
            }
        }
    }
}
