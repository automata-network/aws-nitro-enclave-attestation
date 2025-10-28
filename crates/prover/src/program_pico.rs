use std::fs::{self, File};
use std::marker::PhantomData;
use std::path::PathBuf;

use alloy_primitives::{hex, Bytes, B256, U256};
use alloy_sol_types::SolValue;
use anyhow::anyhow;
use aws_nitro_enclave_attestation_verifier::stub::{
    BatchVerifierInput, BatchVerifierJournal, VerifierInput, VerifierJournal, ZkCoProcessorType,
};
use lazy_static::lazy_static;
use pico_methods::{PICO_AGGREGATOR_ELF, PICO_VERIFIER_ELF};
use p3_field::PrimeField;
use pico_sdk::{client::KoalaBearProverClient, HashableKey};
use pico_vm::{
    configs::stark_config::KoalaBearPoseidon2,
    machine::{
        keys::BaseVerifyingKey,
        proof::MetaProof,
    },
    emulator::stdin::EmulatorStdinBuilder,
};

use crate::{
    program::{Program, RemoteProverConfig},
    RawProof, RawProofType,
};

lazy_static! {
    pub static ref PICO_PROGRAM_VERIFIER: ProgramPico<VerifierInput, VerifierJournal> =
        ProgramPico::new(PICO_VERIFIER_ELF);
    pub static ref PICO_PROGRAM_AGGREGATOR: ProgramPico<BatchVerifierInput, BatchVerifierJournal> =
        ProgramPico::new(PICO_AGGREGATOR_ELF);
}

#[derive(Debug, Clone)]
pub struct PicoProverConfig {
    // Empty for now, extensible for future options (e.g., backend selection)
}

impl Default for PicoProverConfig {
    fn default() -> Self {
        PicoProverConfig {}
    }
}

#[derive(Clone)]
pub struct ProgramPico<Input, Output> {
    elf: &'static [u8],
    _marker: PhantomData<(Input, Output)>,
}

impl<Input, Output> ProgramPico<Input, Output> {
    pub fn new(elf: &'static [u8]) -> Self {
        ProgramPico {
            elf,
            _marker: PhantomData,
        }
    }

    pub fn gen_raw_proof(
        &self,
        stdin_builder: EmulatorStdinBuilder<Vec<u8>, KoalaBearPoseidon2>,
        raw_proof_type: RawProofType,
    ) -> anyhow::Result<RawProof> {
        let client = KoalaBearProverClient::new(self.elf);
        let vk = client.riscv_vk();

        match raw_proof_type {
            RawProofType::Composite => {
                // prove_combine returns (riscv_proof, combine_proof)
                let (riscv_proof, combine_proof) = client.prove_combine(stdin_builder)?;

                // Extract journal from public values
                let journal: Bytes = riscv_proof.pv_stream.unwrap().into();

                RawProof::from_proof(&(combine_proof, vk), journal)
            }
            RawProofType::Groth16 => {
                // Use permanent artifacts directory
                let output_path = PathBuf::from("evm_proof_artifacts");
                std::fs::create_dir_all(&output_path)?;

                // Check if setup is needed (vm_pk doesn't exist)
                let vm_pk_path = output_path.join("vm_pk");
                let need_setup = !vm_pk_path.exists();

                // Prove with EVM backend (KoalaBear)
                client.prove_evm(stdin_builder, need_setup, &output_path, "kb")?;

                // Read proof.json - first 8 elements of 32-byte values
                let proof_file = output_path.join("proof.json");
                let proof_data: Vec<String> = serde_json::from_reader(File::open(proof_file)?)?;
                let proof_bytes: Vec<u8> = proof_data[..8]
                    .iter()
                    .flat_map(|s| {
                        hex::decode(s.trim_start_matches("0x"))
                            .expect("Failed to decode proof hex string")
                    })
                    .collect();

                // Get journal from public values file
                let pv_file = output_path.join("pv_file");
                let journal: Bytes = hex::decode(fs::read_to_string(pv_file)?)?.into();

                RawProof::from_proof(&(proof_bytes, vk), journal)
            }
        }
    }
}

impl<Input, Output> Program for ProgramPico<Input, Output>
where
    Input: SolValue + Send + Sync,
    Output: SolValue + Send + Sync,
{
    type Input = Input;
    type Output = Output;

    fn version(&self) -> &'static str {
        "v1.1.6"
    }

    fn zktype(&self) -> ZkCoProcessorType {
        ZkCoProcessorType::Pico
    }

    fn onchain_proof(&self, proof: &RawProof) -> anyhow::Result<Bytes> {
        let (proof, _) = proof.decode_proof::<(Vec<u8>, BaseVerifyingKey<KoalaBearPoseidon2>)>()?;
        // Decode the 8 * 32-byte proof elements
        let proof_elements: Vec<U256> = proof
            .chunks(32)
            .take(8)
            .map(|chunk| U256::from_be_slice(chunk))
            .collect();

        // ABI encode as uint256[8]
        let proof_array: [U256; 8] = proof_elements.try_into().map_err(|_| anyhow!("Expected exactly 8 proof elements"))?;
        Ok(proof_array.abi_encode().into())
    }

    fn upload_image(&self, _cfg: &RemoteProverConfig) -> anyhow::Result<()> {
        Err(anyhow!("Remote prover is not supported for Pico zkVM"))
    }

    fn program_id(&self) -> B256 {
        let client = KoalaBearProverClient::new(self.elf);
        let vk = client.riscv_vk();
        let vk_digest_bn254 = vk.hash_bn254();
        let vk_bytes = vk_digest_bn254.as_canonical_biguint().to_bytes_be();
        let mut result = [0u8; 32];
        result[1..].copy_from_slice(&vk_bytes);
        B256::from(result)
    }

    fn verify_proof_id(&self) -> B256 {
        let client = KoalaBearProverClient::new(self.elf);
        let vk = client.riscv_vk();
        let vk_digest: [u32; 8] = vk.hash_u32();
        B256::new(unsafe { std::mem::transmute(vk_digest) })
    }

    fn gen_proof(
        &self,
        input: &Self::Input,
        raw_proof_type: RawProofType,
        encoded_composite_proofs: Option<&[&Bytes]>,
    ) -> anyhow::Result<RawProof> {
        let client = KoalaBearProverClient::new(self.elf);
        let mut stdin_builder = client.new_stdin_builder();

        // Write input
        stdin_builder.write_slice(&input.abi_encode());

        // Handle composite proof assumptions for aggregation
        if let Some(encoded_composite_proofs) = encoded_composite_proofs {
            for proof_bytes in encoded_composite_proofs {
                let (combine_proof, vk) = bincode::deserialize::<(MetaProof<KoalaBearPoseidon2>, BaseVerifyingKey<KoalaBearPoseidon2>)>(&proof_bytes)?;
                stdin_builder.write_pico_proof(combine_proof, vk);
            }
        }

        self.gen_raw_proof(stdin_builder, raw_proof_type)
    }
}
