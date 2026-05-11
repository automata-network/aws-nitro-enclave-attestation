use std::collections::BTreeMap;

use anyhow::{anyhow, Context};
use serde::Deserialize;
use serde_bytes::{ByteArray, ByteBuf};
use x509_verifier_rust_crypto::{CertChain, SigAlgo};

use crate::CoseSign1;

#[derive(Debug)]
pub struct AttestationReport {
    doc: AttestationDocument,
    cose_sign: CoseSign1,
}

impl AttestationReport {
    pub fn parse(document_data: &[u8]) -> anyhow::Result<Self> {
        let cose_sign = CoseSign1::from_bytes(document_data)
            .with_context(|| "AttestationDocument::authenticate parse failed")?;
        // Step 2. Exract the attestation document from the COSE_Sign1 structure
        let doc: AttestationDocument = serde_cbor::from_slice(&cose_sign.payload)
            .map_err(|err| anyhow!("document parse failed: {:?}", err))?;

        Ok(Self { doc, cose_sign })
    }

    pub fn cert_chain(&self) -> anyhow::Result<CertChain<'_>> {
        let mut cert_chain = CertChain::new();
        for cert in &self.doc.cabundle {
            cert_chain.add_cert_by_der(cert)?;
        }
        cert_chain.add_cert_by_der(&self.doc.certificate)?;

        Ok(cert_chain)
    }

    pub fn doc(&self) -> &AttestationDocument {
        &self.doc
    }

    /// Following the steps here: <https://docs.aws.amazon.com/enclaves/latest/user/verify-root.html>
    pub fn authenticate(
        &self,
        trusted_certs_len: usize,
        timestamp: u64,
    ) -> anyhow::Result<CertChain<'_>> {
        let cert_chain = self.cert_chain()?;
        match cert_chain.verify_chain(trusted_certs_len) {
            Ok(true) => {}
            Ok(false) => return Err(anyhow!("failed to verify x509 chain")),
            Err(err) => return Err(anyhow!("failed to verify x509 chain: {:?}", err)),
        };
        cert_chain.check_valid(timestamp)?;

        let pubkey = cert_chain.leaf_pubkey();
        let sig_algo = SigAlgo::EcdsaSHA384;

        let result = self.cose_sign.verify_signature(sig_algo, pubkey)?;
        if !result {
            return Err(anyhow!(
                "AttestationDocument::authenticate invalid COSE certificate for provided key"
            ));
        }

        return Ok(cert_chain);
    }
}

#[derive(Debug, Deserialize)]
pub struct AttestationDocument {
    pub module_id: String,
    pub timestamp: u64,
    pub digest: String,
    pub pcrs: BTreeMap<u64, ByteArray<48>>,
    pub certificate: ByteBuf,
    pub cabundle: Vec<ByteBuf>,
    pub public_key: Option<ByteBuf>,
    pub user_data: Option<ByteBuf>,
    pub nonce: Option<ByteBuf>,
}
