use std::fmt::Display;

use alloy_primitives::{Bytes, B128, B256};
use alloy_sol_types::SolValue;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteArray;
use x509_verifier_rust_crypto::sha256;

alloy_sol_types::sol! {
    #[sol(docs, extra_derives(Debug, Serialize, Deserialize))]
    "../../contracts/src/interfaces/INitroEnclaveVerifier.sol"
}

impl Display for Bytes48 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_bytes())
    }
}

impl From<&ByteArray<48>> for Bytes48 {
    fn from(input: &ByteArray<48>) -> Self {
        Self {
            first: B256::from_slice(&input[..32]),
            second: B128::from_slice(&input[32..]),
        }
    }
}

impl Bytes48 {
    pub fn is_zero(&self) -> bool {
        self.first.is_zero() && self.second.is_zero()
    }
    pub fn to_bytes(&self) -> Bytes {
        [self.first.as_slice(), self.second.as_slice()].concat().into()
    }
}

impl VerifierInput {
    pub fn encode(&self) -> Vec<u8> {
        self.abi_encode()
    }

    pub fn decode(buf: &[u8]) -> anyhow::Result<Self> {
        Ok(VerifierInput::abi_decode(buf)
            .map_err(|err| anyhow!("Failed to decode VerifierInput: {}", err))?)
    }
}

impl VerifierJournal {
    pub fn encode(&self) -> Vec<u8> {
        self.abi_encode()
    }

    pub fn digest(&self) -> B256 {
        sha256(&self.encode())
    }

    pub fn decode(buf: &[u8]) -> anyhow::Result<Self> {
        Ok(Self::abi_decode(buf)
            .map_err(|err| anyhow!("Failed to decode VerifierJournal: {}", err))?)
    }
}

impl BatchVerifierJournal {
    pub fn encode(&self) -> Vec<u8> {
        self.abi_encode()
    }

    pub fn decode(buf: &[u8]) -> anyhow::Result<Self> {
        Ok(Self::abi_decode(buf)
            .map_err(|err| anyhow!("Failed to decode BatchVerifierJournal: {}", err))?)
    }
}
