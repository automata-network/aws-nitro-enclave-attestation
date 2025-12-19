use std::sync::Arc;

use alloy_network::{Ethereum, EthereumWallet, TransactionBuilder};
use alloy_primitives::{Address, Bytes, B256};
use alloy_provider::{PendingTransactionBuilder, Provider, ProviderBuilder};
use alloy_rpc_types::TransactionRequest;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::SolCall;
use anyhow::{anyhow, Context};
use aws_nitro_enclave_attestation_verifier::stub::{
    INitroEnclaveVerifier::*, VerifierJournal, ZkCoProcessorConfig, ZkCoProcessorType,
};

use crate::{OnchainProof, ProofType};

#[derive(Debug, Clone)]
pub enum OnchainProofVerifyResult {
    Single(VerifierJournal),
    Batch(Vec<VerifierJournal>),
}

#[derive(Clone)]
pub struct NitroEnclaveVerifierContract {
    contract: Address,
    client: Arc<Box<dyn Provider>>,
}

impl NitroEnclaveVerifierContract {
    pub fn dial(
        endpoint: &str,
        contract: Address,
        private_key: Option<&str>,
    ) -> anyhow::Result<Self> {
        let url = endpoint.try_into()?;

        let provider: Box<dyn Provider> = match private_key {
            Some(pk) => {
                let signer = pk.parse::<PrivateKeySigner>()?;
                let wallet = EthereumWallet::new(signer);
                let provider = ProviderBuilder::new().wallet(wallet).connect_http(url);
                Box::new(provider)
            }
            None => {
                let provider = ProviderBuilder::new().connect_http(url);
                Box::new(provider)
            }
        };

        Ok(Self {
            contract,
            client: Arc::new(provider),
        })
    }

    pub async fn call<T: SolCall>(&self, call: &T) -> anyhow::Result<T::Return> {
        let tx = TransactionRequest::default()
            .with_call(call)
            .to(self.contract);
        let result = self
            .client
            .call(tx)
            .await
            .with_context(|| format!("contract={:?}", self.contract))?;
        let result = T::abi_decode_returns(&result)?;
        Ok(result)
    }

    pub async fn transact<T: SolCall>(
        &self,
        call: &T,
    ) -> anyhow::Result<PendingTransactionBuilder<Ethereum>> {
        let tx = TransactionRequest::default()
            .with_call(call)
            .to(self.contract);
        let result = self.client.send_transaction(tx).await?;
        Ok(result)
    }

    pub async fn verify_proof(
        &self,
        proof: &OnchainProof,
    ) -> anyhow::Result<OnchainProofVerifyResult> {
        if proof.onchain_proof.len() == 0 {
            return Err(anyhow!(
                "Proof does not contain an on-chain proof, unable to verify on-chain."
            ));
        }
        let journal = proof.raw_proof.journal.clone();
        let proof_bytes = proof.onchain_proof.clone();
        let zk = proof.zktype;
        let zk_config = self.zk_config(proof.zktype).await?;
        let verifier_proof_id = self.get_verifier_proof_id(proof.zktype, zk_config.verifierId).await?;
        proof.program_id.verify(&zk_config, verifier_proof_id)?;

        Ok(match proof.proof_type {
            ProofType::Verifier => {
                OnchainProofVerifyResult::Single(self.verify(zk, proof_bytes, journal).await?)
            }
            ProofType::Aggregator => {
                OnchainProofVerifyResult::Batch(self.batch_verify(zk, proof_bytes, journal).await?)
            }
        })
    }

    pub async fn verify(
        &self,
        zk: ZkCoProcessorType,
        proof: Bytes,
        journal: Bytes,
    ) -> anyhow::Result<VerifierJournal> {
        let call = verifyCall {
            output: journal.clone(),
            zkCoprocessor: zk,
            proofBytes: proof.clone(),
        };
        Ok(self
            .call(&call)
            .await
            .with_context(|| format!("proof: {}, journal: {}", proof, journal))?)
    }

    pub async fn batch_verify(
        &self,
        zk: ZkCoProcessorType,
        proof: Bytes,
        journal: Bytes,
    ) -> anyhow::Result<Vec<VerifierJournal>> {
        let call = batchVerifyCall {
            output: journal,
            zkCoprocessor: zk,
            proofBytes: proof,
        };
        Ok(self.call(&call).await?)
    }

    pub async fn root_cert(&self) -> anyhow::Result<B256> {
        Ok(self.call(&rootCertCall {}).await?)
    }

    pub async fn zk_config(&self, zk: ZkCoProcessorType) -> anyhow::Result<ZkCoProcessorConfig> {
        let call = getZkConfigCall { _zkCoProcessor: zk };
        self.call(&call).await
    }

    pub async fn get_verifier_proof_id(
        &self,
        zk: ZkCoProcessorType,
        verifier_id: B256,
    ) -> anyhow::Result<B256> {
        let call = getVerifierProofIdCall {
            _zkCoProcessor: zk,
            _verifierId: verifier_id,
        };
        self.call(&call).await
    }

    pub async fn max_time_diff(&self) -> anyhow::Result<u64> {
        Ok(self.call(&maxTimeDiffCall {}).await?)
    }

    pub async fn batch_query_cert_cache(
        &self,
        certs_digests: Vec<Vec<B256>>,
    ) -> anyhow::Result<Vec<u8>> {
        if certs_digests.is_empty() {
            return Ok(vec![]);
        }

        for report_certs in &certs_digests {
            let len = report_certs.len();
            if len == 0 || len > 8 {
                return Err(anyhow!(
                    "Too many certificate chains provided, maximum is 8, got: {len}"
                ));
            }
        }

        let result = self
            .call(&checkTrustedIntermediateCertsCall {
                _report_certs: certs_digests,
            })
            .await?;
        Ok(result)
    }
}
