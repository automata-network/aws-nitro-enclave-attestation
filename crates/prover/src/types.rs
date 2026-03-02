use alloy_primitives::{Bytes, B256};
use alloy_sol_types::{SolType, SolValue};
use anyhow::anyhow;
use aws_nitro_enclave_attestation_verifier::stub::{ZkCoProcessorConfig, ZkCoProcessorType};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::program::Program;

#[derive(Debug, Serialize, Deserialize)]
pub struct OnchainProof {
    pub zktype: ZkCoProcessorType,
    pub zkvm_version: String,
    pub program_id: ProgramId,
    pub raw_proof: RawProof,
    pub onchain_proof: Bytes,
    pub proof_type: ProofType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ProofType {
    Verifier,
    Aggregator,
}

impl OnchainProof {
    pub fn new(
        zktype: ZkCoProcessorType,
        zkvm_version: String,
        program_id: ProgramId,
        onchain_proof: Bytes,
        raw_proof: RawProof,
        proof_type: ProofType,
    ) -> Self {
        Self {
            zktype,
            zkvm_version,
            program_id,
            raw_proof,
            onchain_proof,
            proof_type,
        }
    }

    pub fn new_from_program<P: Program + ?Sized>(
        p: &P,
        program_id: ProgramId,
        raw_proof: RawProof,
        proof_type: ProofType,
    ) -> anyhow::Result<Self> {
        Ok(Self::new(
            p.zktype(),
            p.version().into(),
            program_id,
            p.onchain_proof(&raw_proof)?,
            raw_proof,
            proof_type,
        ))
    }

    pub fn encode_json(&self) -> anyhow::Result<Vec<u8>> {
        serde_json::to_vec_pretty(self).map_err(|e| anyhow!("Failed to serialize proof: {}", e))
    }

    pub fn decode_json(data: &[u8]) -> anyhow::Result<Self> {
        serde_json::from_slice(data).map_err(|e| anyhow!("Failed to deserialize proof: {}", e))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProgramId {
    pub verifier_id: B256,
    pub verifier_proof_id: B256,
    pub aggregator_id: B256,
}

impl ProgramId {
    pub fn verify(
        &self,
        zk_config: &ZkCoProcessorConfig,
        verifier_proof_id: B256,
    ) -> anyhow::Result<()> {
        if zk_config.aggregatorId != self.aggregator_id
            || zk_config.verifierId != self.verifier_id
            || verifier_proof_id != self.verifier_proof_id
        {
            return Err(anyhow!(
                "Program ID mismatch with on-chain config: want: {{verifierId={}, verifierProofId={}, aggregatorId={}}}, got: {{verifierId={}, verifierProofId={}, aggregatorId={}}})",
                zk_config.verifierId,
                verifier_proof_id,
                zk_config.aggregatorId,
                self.verifier_id,
                self.verifier_proof_id,
                self.aggregator_id,
            ));
        }
        Ok(())
    }

    pub fn encode_json(&self, zk: ZkCoProcessorType) -> anyhow::Result<Vec<u8>> {
        let val = serde_json::to_value(self)?;
        let mut map = serde_json::Map::new();
        map.insert("program_id".into(), val);
        map.insert("zktype".into(), serde_json::to_value(&zk)?);
        serde_json::to_vec_pretty(&map)
            .map_err(|e| anyhow!("Failed to serialize program ID: {}", e))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RawProofType {
    Groth16,
    Composite,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawProof {
    pub encoded_proof: Bytes,
    pub journal: Bytes,
}

impl RawProof {
    pub fn from_proof<P>(proof: P, journal: Bytes) -> anyhow::Result<Self>
    where
        P: Serialize,
    {
        let encoded_proof = bincode::serialize(&proof)?.into();
        Ok(Self {
            journal,
            encoded_proof,
        })
    }

    pub fn decode_proof<P>(&self) -> anyhow::Result<P>
    where
        P: Serialize + DeserializeOwned,
    {
        bincode::deserialize(&self.encoded_proof)
            .map_err(|err| anyhow!("Failed to deserialize proof: {}", err))
    }

    pub fn decode_journal<J>(&self) -> anyhow::Result<J>
    where
        J: SolValue + From<<<J as SolValue>::SolType as SolType>::RustType>,
    {
        J::abi_decode(&self.journal).map_err(|err| anyhow!("Failed to decode journal: {}", err))
    }
}
