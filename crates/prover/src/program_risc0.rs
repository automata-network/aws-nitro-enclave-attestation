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
    request_builder::OfferParams,
    storage::storage_provider_from_env,
    Deployment,
    StorageProvider,
};
use lazy_static::lazy_static;
use risc0_ethereum_contracts::groth16;
use risc0_methods::{
    RISC0_AGGREGATOR_ELF, RISC0_AGGREGATOR_ID, RISC0_VERIFIER_ELF, RISC0_VERIFIER_ID,
};
use risc0_zkvm::{default_executor, Digest, ExecutorEnv, InnerReceipt, VERSION};

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
    /// Optional program URL for pre-uploaded ELF (env: BOUNDLESS_PROGRAM_URL)
    pub program_url: Option<String>,
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
            program_url: std::env::var("BOUNDLESS_PROGRAM_URL").ok(),
            proof_type: BoundlessProofType::default(),
            min_price: None,
            max_price: None,
            timeout: None,
            ramp_up_period: None,
        }
    }
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

    /// Production: generate proof via Boundless network
    fn gen_proof_boundless(
        &self,
        input_bytes: &[u8],
        cfg: &RiscZeroProverConfig,
    ) -> anyhow::Result<RawProof> {
        let rpc_url = cfg
            .rpc_url
            .as_ref()
            .ok_or_else(|| anyhow!("missing BOUNDLESS_RPC_URL"))?;
        let private_key_hex = cfg
            .private_key
            .as_ref()
            .ok_or_else(|| anyhow!("missing BOUNDLESS_PRIVATE_KEY"))?;

        block_on(async {
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

            // Get storage provider from env (PINATA_JWT, IPFS_GATEWAY_URL)
            let storage_provider = storage_provider_from_env()
                .context("Failed to get storage provider (check PINATA_JWT env var)")?;

            let client = BoundlessClient::builder()
                .with_rpc_url(rpc_url_parsed)
                .with_deployment(deployment)
                .with_storage_provider(Some(storage_provider))
                .with_private_key(private_key)
                .config_offer_layer(|config| {
                    config
                        .max_price_per_cycle(parse_units("0.001", "gwei").unwrap())
                        .min_price_per_cycle(parse_units("0.0001", "gwei").unwrap())
                })
                .build()
                .await
                .context("Failed to build Boundless client")?;

            // Build request using client.new_request()
            let mut request_builder = client.new_request().with_stdin(input_bytes);

            // Set program (ELF or URL)
            if let Some(ref program_url) = cfg.program_url {
                request_builder = request_builder
                    .with_program_url(program_url.as_str())
                    .context("Failed to set program URL")?;
            } else {
                request_builder = request_builder.with_program(self.elf.to_vec());
            }

            // Set proof type
            match cfg.proof_type {
                BoundlessProofType::Groth16 => {
                    request_builder = request_builder.with_groth16_proof();
                }
                BoundlessProofType::Merkle => {
                    // Merkle is the default, no flag needed
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
            if let Some(timeout) = cfg.timeout {
                offer_builder.timeout(timeout);
            }
            if let Some(ramp_up_period) = cfg.ramp_up_period {
                offer_builder.ramp_up_period(ramp_up_period);
            }
            let collateral_amount = parse_units("10", "ether").unwrap();
            offer_builder.lock_collateral(collateral_amount);
            request_builder = request_builder.with_offer(offer_builder.build()?);

            // Submit and wait for fulfillment
            let (request_id, expires_at) = client
                .submit_onchain(request_builder)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to submit proof request: {:?}", e))?;

            tracing::info!("Boundless request submitted: {:x}", request_id);

            let fulfillment = client
                .wait_for_request_fulfillment(request_id, Duration::from_secs(5), expires_at)
                .await
                .context("Failed waiting for proof fulfillment")?;

            // Convert Boundless response to RawProof
            // fulfillmentData contains the journal bytes
            let seal_bytes = fulfillment.seal.to_vec();
            let journal: Bytes = fulfillment.fulfillmentData.to_vec().into();

            Ok(RawProof {
                encoded_proof: seal_bytes.into(),
                journal,
            })
        })
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
        let receipt = proof.decode_proof::<InnerReceipt>()?;
        let encoded_proof = match receipt {
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

    fn gen_proof(
        &self,
        input: &Self::Input,
        _raw_proof_type: RawProofType,
        _encoded_composite_proofs: Option<&[&Bytes]>,
    ) -> anyhow::Result<RawProof> {
        let dev_mode = std::env::var("RISC0_DEV_MODE")
            .map(|v| v == "1")
            .unwrap_or(false);

        let input_bytes = input.abi_encode();

        if dev_mode {
            // Dev mode: run executor only (no proof generation)
            self.gen_dev_proof(&input_bytes)
        } else {
            // Production: use Boundless network
            let cfg = RiscZeroProverConfig::default();
            self.gen_proof_boundless(&input_bytes, &cfg)
        }
    }
}
