use std::marker::PhantomData;

use alloy_primitives::{hex::FromHex, Bytes, B256};
use alloy_sol_types::SolValue;
use anyhow::anyhow;
use aws_nitro_enclave_attestation_verifier::stub::{
    BatchVerifierJournal, VerifierInput, VerifierJournal, ZkCoProcessorType,
};
use lazy_static::lazy_static;
use sp1_methods::{
    ENV_PROVER, SP1_AGGREGATOR_ELF, SP1_AGGREGATOR_PK, SP1_AGGREGATOR_VK, SP1_VERIFIER_ELF,
    SP1_VERIFIER_PK, SP1_VERIFIER_VK,
};
use sp1_sdk::{
    network::builder::NetworkProverBuilder, HashableKey, SP1Proof, SP1ProvingKey, SP1Stdin,
    SP1VerifyingKey, SP1_CIRCUIT_VERSION,
};

use crate::{
    program::{Program, RemoteProverConfig},
    utils::block_on,
    RawProof, RawProofType,
};

#[derive(Debug, Clone)]
pub struct SP1ProverConfig {
    pub private_key: Option<String>,
    pub rpc_url: Option<String>,
}

impl Default for SP1ProverConfig {
    fn default() -> Self {
        SP1ProverConfig {
            private_key: std::env::var("NETWORK_PRIVATE_KEY").ok(),
            rpc_url: std::env::var("NETWORK_RPC_URL").ok(),
        }
    }
}

impl TryFrom<SP1ProverConfig> for RemoteProverConfig {
    type Error = anyhow::Error;
    fn try_from(value: SP1ProverConfig) -> anyhow::Result<Self> {
        Ok(RemoteProverConfig {
            api_url: value.rpc_url,
            api_key: value
                .private_key
                .ok_or_else(|| anyhow!("missing private key"))?,
        })
    }
}

lazy_static! {
    pub static ref SP1_PROGRAM_VERIFIER: ProgramSP1<VerifierInput, VerifierJournal> =
        ProgramSP1::new(SP1_VERIFIER_ELF, &SP1_VERIFIER_VK, &SP1_VERIFIER_PK);
    pub static ref SP1_PROGRAM_AGGREGATOR: ProgramSP1<BatchVerifierJournal, BatchVerifierJournal> =
        ProgramSP1::new(SP1_AGGREGATOR_ELF, &SP1_AGGREGATOR_VK, &SP1_AGGREGATOR_PK);
}

#[derive(Clone)]
pub struct ProgramSP1<Input, Output> {
    vk: &'static SP1VerifyingKey,
    pk: &'static SP1ProvingKey,
    elf: &'static [u8],
    _marker: PhantomData<(Input, Output)>,
}

impl<Input, Output> ProgramSP1<Input, Output> {
    pub fn new(
        elf: &'static [u8],
        vk: &'static SP1VerifyingKey,
        pk: &'static SP1ProvingKey,
    ) -> Self {
        ProgramSP1 {
            vk,
            pk,
            elf,
            _marker: PhantomData,
        }
    }

    fn gen_raw_proof(
        &self,
        stdin: SP1Stdin,
        raw_proof_type: RawProofType,
    ) -> anyhow::Result<RawProof> {
        let prover = ENV_PROVER.prove(&self.pk, &stdin);
        let prover = match raw_proof_type {
            RawProofType::Composite => prover.compressed(),
            RawProofType::Groth16 => prover.groth16(),
        };
        let proof = prover.run()?;

        Ok(RawProof::from_proof(
            &(proof.proof, self.vk),
            proof.public_values.to_vec().into(),
        )?)
    }
}

impl<Input, Output> Program for ProgramSP1<Input, Output>
where
    Input: SolValue + Send + Sync,
    Output: SolValue + Send + Sync,
{
    type Input = Input;
    type Output = Output;
    fn version(&self) -> &'static str {
        SP1_CIRCUIT_VERSION
    }
    fn zktype(&self) -> ZkCoProcessorType {
        ZkCoProcessorType::Succinct
    }
    fn onchain_proof(&self, proof: &RawProof) -> anyhow::Result<Bytes> {
        let (sp1_proof, _) = proof.decode_proof::<(SP1Proof, SP1VerifyingKey)>()?;
        Ok(match sp1_proof {
            SP1Proof::Groth16(groth16_proof) => {
                if groth16_proof.encoded_proof.is_empty() {
                    return Ok(Bytes::new());
                }
                let proof_bytes = Bytes::from_hex(&groth16_proof.encoded_proof)?;
                let proof: Bytes = [
                    groth16_proof.groth16_vkey_hash[..4].to_vec(),
                    proof_bytes.to_vec(),
                ]
                .concat()
                .into();
                proof
            }
            SP1Proof::Plonk(plonk_proof) => {
                if plonk_proof.encoded_proof.is_empty() {
                    return Ok(Bytes::new());
                }
                let proof_bytes = Bytes::from_hex(&plonk_proof.encoded_proof)?;
                let proof: Bytes = [
                    plonk_proof.plonk_vkey_hash[..4].to_vec(),
                    proof_bytes.to_vec(),
                ]
                .concat()
                .into();
                proof
            }
            SP1Proof::Compressed(_) | SP1Proof::Core(_) => Bytes::new(),
        })
    }

    fn upload_image(&self, cfg: &RemoteProverConfig) -> anyhow::Result<()> {
        block_on(async {
            let mut builder = NetworkProverBuilder::default().private_key(&cfg.api_key);
            if let Some(api_url) = &cfg.api_url {
                builder = builder.rpc_url(&api_url);
            }
            let prover = builder.build();
            prover.register_program(&self.vk, self.elf).await?;
            Ok(())
        })
    }

    fn program_id(&self) -> B256 {
        self.vk.bytes32_raw().into()
    }

    fn verify_proof_id(&self) -> B256 {
        B256::new(unsafe { std::mem::transmute(self.vk.hash_u32()) })
    }

    fn gen_proof(
        &self,
        input: &Self::Input,
        raw_proof_type: RawProofType,
        encoded_composite_proofs: Option<&[&Bytes]>,
    ) -> anyhow::Result<RawProof> {
        let mut stdin = SP1Stdin::new();
        stdin.write_vec(input.abi_encode());
        if let Some(encoded_composite_proofs) = encoded_composite_proofs {
            for proof in encoded_composite_proofs {
                let (proof, vk) = bincode::deserialize::<(SP1Proof, SP1VerifyingKey)>(&proof)?;
                let SP1Proof::Compressed(proof) = proof else {
                    return Err(anyhow!("Expected a compressed SP1 proof"));
                };
                stdin.write_proof(*proof, vk.vk);
            }
        }
        Ok(self.gen_raw_proof(stdin, raw_proof_type)?)
    }
}
