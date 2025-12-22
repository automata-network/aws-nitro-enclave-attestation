//! Serializable wrapper types for WASM boundary.
//!
//! These types convert the alloy-sol-types generated structures into
//! frontend-friendly, JSON-serializable formats.

use alloy_primitives::{FixedBytes, B128, B256};
use serde::{Deserialize, Serialize};

use crate::{BatchVerifierJournal, Bytes48, Pcr, VerificationResult, VerifierJournal};

/// Wrapper for PCR (Platform Configuration Register) entry.
///
/// PCRs contain cryptographic measurements of the enclave's runtime state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PcrWrapper {
    /// PCR index number (0-23 for AWS Nitro Enclaves)
    pub index: u64,

    /// 48-byte PCR measurement value (SHA-384 hash), hex-encoded with "0x" prefix
    pub value: String,
}

impl From<Pcr> for PcrWrapper {
    fn from(pcr: Pcr) -> Self {
        // Combine first (32 bytes) and second (16 bytes) into 48-byte value
        let mut bytes = Vec::with_capacity(48);
        bytes.extend_from_slice(pcr.value.first.as_slice());
        bytes.extend_from_slice(pcr.value.second.as_slice());

        Self {
            index: pcr.index,
            value: format!("0x{}", hex::encode(bytes)),
        }
    }
}

impl TryFrom<PcrWrapper> for Pcr {
    type Error = anyhow::Error;

    fn try_from(wrapper: PcrWrapper) -> Result<Self, Self::Error> {
        let bytes = hex::decode(wrapper.value.trim_start_matches("0x"))
            .map_err(|e| anyhow::anyhow!("Failed to decode PCR value: {}", e))?;

        if bytes.len() != 48 {
            anyhow::bail!(
                "Invalid PCR value length: expected 48 bytes, got {}",
                bytes.len()
            );
        }

        Ok(Self {
            index: wrapper.index,
            value: Bytes48 {
                first: B256::from_slice(&bytes[..32]),
                second: B128::from_slice(&bytes[32..]),
            },
        })
    }
}

/// Wrapper for VerifierJournal that can be serialized to/from JSON for WASM.
///
/// All byte arrays are hex-encoded strings for JavaScript compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifierJournalWrapper {
    /// Verification result as a string: "Success", "RootCertNotTrusted",
    /// "IntermediateCertsNotTrusted", or "InvalidTimestamp"
    pub result: String,

    /// Number of trusted certificates in the chain
    pub trusted_certs_prefix_len: u8,

    /// Attestation timestamp (Unix timestamp in milliseconds)
    pub timestamp: u64,

    /// Certificate hashes (bytes32[]), each hex-encoded with "0x" prefix
    pub certs: Vec<String>,

    /// User-defined data embedded in the attestation, hex-encoded with "0x" prefix
    pub user_data: String,

    /// Cryptographic nonce used for replay protection, hex-encoded with "0x" prefix
    pub nonce: String,

    /// Public key extracted from the attestation, hex-encoded with "0x" prefix
    pub public_key: String,

    /// Platform Configuration Registers (integrity measurements)
    pub pcrs: Vec<PcrWrapper>,

    /// AWS Nitro Enclave module identifier
    pub module_id: String,
}

impl From<VerifierJournal> for VerifierJournalWrapper {
    fn from(journal: VerifierJournal) -> Self {
        let result = match journal.result {
            VerificationResult::Success => "Success",
            VerificationResult::RootCertNotTrusted => "RootCertNotTrusted",
            VerificationResult::IntermediateCertsNotTrusted => "IntermediateCertsNotTrusted",
            VerificationResult::InvalidTimestamp => "InvalidTimestamp",
            _ => "Unknown",
        }
        .to_string();

        let certs = journal
            .certs
            .iter()
            .map(|c| format!("0x{}", hex::encode(c)))
            .collect();

        let user_data = format!("0x{}", hex::encode(&journal.userData));
        let nonce = format!("0x{}", hex::encode(&journal.nonce));
        let public_key = format!("0x{}", hex::encode(&journal.publicKey));

        let pcrs = journal.pcrs.into_iter().map(PcrWrapper::from).collect();

        Self {
            result,
            trusted_certs_prefix_len: journal.trustedCertsPrefixLen,
            timestamp: journal.timestamp,
            certs,
            user_data,
            nonce,
            public_key,
            pcrs,
            module_id: journal.moduleId,
        }
    }
}

impl TryFrom<VerifierJournalWrapper> for VerifierJournal {
    type Error = anyhow::Error;

    fn try_from(wrapper: VerifierJournalWrapper) -> Result<Self, Self::Error> {
        let result = match wrapper.result.as_str() {
            "Success" => VerificationResult::Success,
            "RootCertNotTrusted" => VerificationResult::RootCertNotTrusted,
            "IntermediateCertsNotTrusted" => VerificationResult::IntermediateCertsNotTrusted,
            "InvalidTimestamp" => VerificationResult::InvalidTimestamp,
            _ => anyhow::bail!("Unknown verification result: {}", wrapper.result),
        };

        let certs: Result<Vec<FixedBytes<32>>, _> = wrapper
            .certs
            .iter()
            .map(|c| {
                let bytes = hex::decode(c.trim_start_matches("0x"))?;
                if bytes.len() != 32 {
                    anyhow::bail!(
                        "Invalid cert hash length: expected 32 bytes, got {}",
                        bytes.len()
                    );
                }
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&bytes);
                Ok(FixedBytes(arr))
            })
            .collect();
        let certs = certs?;

        let user_data = hex::decode(wrapper.user_data.trim_start_matches("0x"))
            .map_err(|e| anyhow::anyhow!("Failed to decode userData: {}", e))?;

        let nonce = hex::decode(wrapper.nonce.trim_start_matches("0x"))
            .map_err(|e| anyhow::anyhow!("Failed to decode nonce: {}", e))?;

        let public_key = hex::decode(wrapper.public_key.trim_start_matches("0x"))
            .map_err(|e| anyhow::anyhow!("Failed to decode publicKey: {}", e))?;

        let pcrs: Result<Vec<Pcr>, _> = wrapper.pcrs.into_iter().map(Pcr::try_from).collect();
        let pcrs = pcrs?;

        Ok(Self {
            result,
            trustedCertsPrefixLen: wrapper.trusted_certs_prefix_len,
            timestamp: wrapper.timestamp,
            certs,
            userData: user_data.into(),
            nonce: nonce.into(),
            publicKey: public_key.into(),
            pcrs,
            moduleId: wrapper.module_id,
        })
    }
}

/// Wrapper for BatchVerifierJournal that can be serialized to/from JSON for WASM.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchVerifierJournalWrapper {
    /// Verification key that was used for batch verification, hex-encoded with "0x" prefix
    pub verifier_vk: String,

    /// Array of verified attestation results
    pub outputs: Vec<VerifierJournalWrapper>,
}

impl From<BatchVerifierJournal> for BatchVerifierJournalWrapper {
    fn from(batch: BatchVerifierJournal) -> Self {
        Self {
            verifier_vk: format!("0x{}", hex::encode(batch.verifierVk)),
            outputs: batch
                .outputs
                .into_iter()
                .map(VerifierJournalWrapper::from)
                .collect(),
        }
    }
}

impl TryFrom<BatchVerifierJournalWrapper> for BatchVerifierJournal {
    type Error = anyhow::Error;

    fn try_from(wrapper: BatchVerifierJournalWrapper) -> Result<Self, Self::Error> {
        let vk_bytes = hex::decode(wrapper.verifier_vk.trim_start_matches("0x"))
            .map_err(|e| anyhow::anyhow!("Failed to decode verifierVk: {}", e))?;

        if vk_bytes.len() != 32 {
            anyhow::bail!(
                "Invalid verifierVk length: expected 32 bytes, got {}",
                vk_bytes.len()
            );
        }

        let mut vk_arr = [0u8; 32];
        vk_arr.copy_from_slice(&vk_bytes);

        let outputs: Result<Vec<VerifierJournal>, _> = wrapper
            .outputs
            .into_iter()
            .map(VerifierJournal::try_from)
            .collect();
        let outputs = outputs?;

        Ok(Self {
            verifierVk: FixedBytes(vk_arr),
            outputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::FixedBytes;

    // Helper to compare VerificationResult variants
    fn result_matches(a: &VerificationResult, b: &VerificationResult) -> bool {
        matches!(
            (a, b),
            (VerificationResult::Success, VerificationResult::Success)
                | (
                    VerificationResult::RootCertNotTrusted,
                    VerificationResult::RootCertNotTrusted
                )
                | (
                    VerificationResult::IntermediateCertsNotTrusted,
                    VerificationResult::IntermediateCertsNotTrusted
                )
                | (
                    VerificationResult::InvalidTimestamp,
                    VerificationResult::InvalidTimestamp
                )
        )
    }

    #[test]
    fn test_pcr_roundtrip() {
        let pcr = Pcr {
            index: 0,
            value: Bytes48 {
                first: B256::from([1u8; 32]),
                second: B128::from([2u8; 16]),
            },
        };

        let wrapper: PcrWrapper = pcr.clone().into();
        let roundtrip: Pcr = wrapper.try_into().expect("roundtrip failed");

        assert_eq!(pcr.index, roundtrip.index);
        assert_eq!(pcr.value.first, roundtrip.value.first);
        assert_eq!(pcr.value.second, roundtrip.value.second);
    }

    #[test]
    fn test_verifier_journal_roundtrip() {
        let journal = VerifierJournal {
            result: VerificationResult::Success,
            trustedCertsPrefixLen: 2,
            timestamp: 1234567890000,
            certs: vec![FixedBytes([1u8; 32]), FixedBytes([2u8; 32])],
            userData: vec![0xaa, 0xbb, 0xcc].into(),
            nonce: vec![0x11, 0x22, 0x33].into(),
            publicKey: vec![0x04; 65].into(),
            pcrs: vec![Pcr {
                index: 0,
                value: Bytes48 {
                    first: B256::from([1u8; 32]),
                    second: B128::from([2u8; 16]),
                },
            }],
            moduleId: "test-module-id".to_string(),
        };

        let wrapper: VerifierJournalWrapper = journal.clone().into();
        let roundtrip: VerifierJournal = wrapper.try_into().expect("roundtrip failed");

        assert!(result_matches(&journal.result, &roundtrip.result));
        assert_eq!(
            journal.trustedCertsPrefixLen,
            roundtrip.trustedCertsPrefixLen
        );
        assert_eq!(journal.timestamp, roundtrip.timestamp);
        assert_eq!(journal.certs, roundtrip.certs);
        assert_eq!(journal.userData, roundtrip.userData);
        assert_eq!(journal.nonce, roundtrip.nonce);
        assert_eq!(journal.publicKey, roundtrip.publicKey);
        assert_eq!(journal.pcrs.len(), roundtrip.pcrs.len());
        assert_eq!(journal.moduleId, roundtrip.moduleId);
    }

    #[test]
    fn test_batch_verifier_journal_roundtrip() {
        let batch = BatchVerifierJournal {
            verifierVk: FixedBytes([0xab; 32]),
            outputs: vec![VerifierJournal {
                result: VerificationResult::Success,
                trustedCertsPrefixLen: 1,
                timestamp: 1234567890000,
                certs: vec![FixedBytes([1u8; 32])],
                userData: vec![].into(),
                nonce: vec![].into(),
                publicKey: vec![].into(),
                pcrs: vec![],
                moduleId: "test".to_string(),
            }],
        };

        let wrapper: BatchVerifierJournalWrapper = batch.clone().into();
        let roundtrip: BatchVerifierJournal = wrapper.try_into().expect("roundtrip failed");

        assert_eq!(batch.verifierVk, roundtrip.verifierVk);
        assert_eq!(batch.outputs.len(), roundtrip.outputs.len());
    }
}
