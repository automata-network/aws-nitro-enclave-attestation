use std::borrow::Cow;

use crate::ec_decode_sig;
use crate::verify_signature;
use crate::KeyAlgo;
use crate::PubKey;
use crate::SigAlgo;

use alloy_primitives::B256;
use anyhow::anyhow;
use anyhow::Context;
use sha2::Sha256;
use x509_parser::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct Cert<'a> {
    pub raw: X509Certificate<'a>,
    pub bytes: &'a [u8],
    pubkey_algo: KeyAlgo,
}

impl<'a> Cert<'a> {
    pub fn parse_der(bytes: &'a [u8]) -> anyhow::Result<Self> {
        let (remain, raw) = X509Certificate::from_der(bytes)
            .map_err(|err| anyhow!("parse cert failed: {:?}", err))?;
        if remain.len() != 0 {
            return Err(anyhow!("parse cert not consume all bytes"));
        }
        let pubkey_algo = {
            let info = raw.public_key();
            KeyAlgo::from_algo(&info.algorithm)?
        };
        Ok(Self {
            raw,
            bytes,
            pubkey_algo,
        })
    }

    pub fn validity(&self) -> (ASN1Time, ASN1Time) {
        let validity = &self.raw.validity;
        (validity.not_before, validity.not_after)
    }

    pub fn check_valid(&self, time: ASN1Time) -> anyhow::Result<()> {
        let validity = &self.raw.validity;
        if !validity.is_valid_at(time) {
            Err(anyhow!(
                "certificate is not valid at time: {}({}), range: {}({}) - {}({})",
                time,
                time.timestamp(),
                validity.not_before,
                validity.not_before.timestamp(),
                validity.not_after,
                validity.not_after.timestamp(),
            ))
        } else {
            Ok(())
        }
    }

    pub fn digest(&self) -> B256 {
        sha256(self.bytes)
    }

    pub fn pubkey_algo(&self) -> anyhow::Result<KeyAlgo> {
        let info = self.raw.public_key();
        Ok(KeyAlgo::from_algo(&info.algorithm)?)
    }

    pub fn sig_algo(&self) -> anyhow::Result<SigAlgo> {
        SigAlgo::from_oid(self.raw.signature_algorithm.oid())
    }

    pub fn pubkey(&self) -> PubKey<'_> {
        PubKey {
            algo: self.pubkey_algo.clone(),
            val: self.raw.public_key().subject_public_key.as_ref(),
        }
    }

    pub fn signature(&self) -> &[u8] {
        self.raw.signature_value.as_ref()
    }

    pub fn tbs_certificate(&self) -> &[u8] {
        self.raw.tbs_certificate.as_ref()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn verify(&self, issuer: Option<&Self>) -> anyhow::Result<bool> {
        let issuer_key = issuer.unwrap_or(self).pubkey();
        let sig_algo = self.sig_algo()?;

        sig_algo.check_compatible_with(issuer_key.algo)?;

        let mut sig = Cow::Borrowed(self.signature());
        if let KeyAlgo::ECDSA(params) = issuer_key.algo {
            sig = Cow::Owned(ec_decode_sig(&sig, params)?);
        }
        let result = verify_signature(issuer_key, sig_algo, &sig, self.tbs_certificate())?;
        Ok(result)
    }
}

pub fn sha256(bytes: &[u8]) -> B256 {
    use sha2::Digest;
    let digest: [u8; 32] = Sha256::digest(bytes).into();
    digest.into()
}

pub struct CertChain<'a> {
    // cert order: root -> leaf
    pub certs: Vec<Cert<'a>>,
    // digest will inherit the digest of parent certs
    pub path_digest: Vec<B256>,
}

impl<'a> CertChain<'a> {
    pub fn new() -> Self {
        Self {
            certs: Vec::new(),
            path_digest: Vec::new(),
        }
    }

    pub fn parse_rev<'b: 'a, I, N>(chain: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = &'b N>,
        I::IntoIter: DoubleEndedIterator,
        N: AsRef<[u8]> + 'b,
    {
        let mut cert_chain = Self::new();
        for cert_der in chain.into_iter().rev() {
            cert_chain.add_cert_by_der(cert_der.as_ref())?;
        }
        Ok(cert_chain)
    }

    pub fn parse<'b: 'a, I, N>(chain: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = &'b N>,
        N: AsRef<[u8]> + 'b,
    {
        let mut cert_chain = Self::new();
        for cert_der in chain {
            cert_chain.add_cert_by_der(cert_der.as_ref())?;
        }
        Ok(cert_chain)
    }

    pub fn add_cert_by_der<'b: 'a>(&mut self, buf: &'b [u8]) -> anyhow::Result<()> {
        let cert = Cert::parse_der(buf)?;
        self.path_digest.push(match self.path_digest.last() {
            Some(parent_digest) => sha256(&[parent_digest, &cert.digest()].concat()),
            None => cert.digest(),
        });
        self.certs.push(cert);
        Ok(())
    }

    pub fn leaf_pubkey(&self) -> PubKey<'_> {
        self.leaf().pubkey()
    }

    pub fn leaf(&self) -> &Cert<'a> {
        &self.certs[self.certs.len() - 1]
    }

    pub fn root(&self) -> &Cert<'a> {
        &self.certs[0]
    }

    pub fn digest(&self) -> &[B256] {
        &self.path_digest
    }

    pub fn check_valid(&self, timestamp: u64) -> anyhow::Result<()> {
        let time = ASN1Time::from_timestamp(timestamp as i64)
            .map_err(|_| anyhow!("invalid timestamp: {}", timestamp))?;
        if self.certs.is_empty() {
            return Err(anyhow!("cert chain is empty"));
        }
        for (idx, cert) in self.certs.iter().enumerate() {
            cert.check_valid(time).with_context(|| {
                format!("cert not valid at chain [{}/{}]", idx + 1, self.certs.len())
            })?;
        }
        Ok(())
    }

    pub fn verify_chain(&self, trusted_certs_len: usize) -> anyhow::Result<bool> {
        if trusted_certs_len > self.certs.len() {
            return Err(anyhow!(
                "trusted certs length is greater than cert chain length"
            ));
        }
        for i in trusted_certs_len..self.certs.len() {
            let subject = &self.certs[i];
            let issuer = if i == 0 {
                None
            } else {
                Some(&self.certs[i - 1])
            };
            if !subject
                .verify(issuer)
                .with_context(|| format!("verify cert sig failed at {}", i))?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
