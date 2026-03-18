//! WASM bindings for AWS Nitro Enclave attestation utilities.
//!
//! Provides functions for parsing and encoding VerifierJournal and
//! VerifierJournal[] data structures for frontend integration.

use alloy_sol_types::SolValue;
use wasm_bindgen::prelude::*;

use crate::{VerifierJournal, VerifierJournalWrapper};

/// Parse ABI-encoded VerifierJournal bytes into a JSON-serializable object.
///
/// # Arguments
/// * `bytes` - ABI-encoded VerifierJournal bytes
///
/// # Returns
/// A JavaScript object representing the parsed VerifierJournal with fields:
/// - `result`: String ("Success", "RootCertNotTrusted", "IntermediateCertsNotTrusted", "InvalidTimestamp")
/// - `trustedCertsPrefixLen`: Number
/// - `timestamp`: Number (Unix epoch milliseconds)
/// - `certs`: Array of Strings (hex-encoded bytes32 values with "0x" prefix)
/// - `userData`: String (hex-encoded with "0x" prefix)
/// - `nonce`: String (hex-encoded with "0x" prefix)
/// - `publicKey`: String (hex-encoded with "0x" prefix)
/// - `pcrs`: Array of objects with `index` (number) and `value` (hex string)
/// - `moduleId`: String
#[wasm_bindgen]
pub fn parse_verified_journal(bytes: &[u8]) -> JsValue {
    let journal = VerifierJournal::decode(bytes).expect("Failed to decode VerifierJournal");
    let wrapper: VerifierJournalWrapper = journal.into();
    serde_wasm_bindgen::to_value(&wrapper).expect("Failed to serialize to JsValue")
}

/// Encode a VerifierJournal object to ABI-encoded bytes.
///
/// # Arguments
/// * `journal` - A JavaScript object with the VerifierJournal structure (see parse_verified_journal for format)
///
/// # Returns
/// ABI-encoded bytes as a Uint8Array
#[wasm_bindgen]
pub fn encode_verified_journal(journal: JsValue) -> Vec<u8> {
    let wrapper: VerifierJournalWrapper =
        serde_wasm_bindgen::from_value(journal).expect("Failed to deserialize from JsValue");
    let journal: VerifierJournal = wrapper
        .try_into()
        .expect("Failed to convert to VerifierJournal");
    journal.encode()
}

/// Parse ABI-encoded VerifierJournal[] bytes into a JSON-serializable array.
///
/// # Arguments
/// * `bytes` - ABI-encoded VerifierJournal[] bytes (from BatchAttestationSubmitted event)
///
/// # Returns
/// A JavaScript array of VerifierJournal objects (see parse_verified_journal for format)
#[wasm_bindgen]
pub fn parse_batch_verified_journal(bytes: &[u8]) -> JsValue {
    let journals =
        <Vec<VerifierJournal>>::abi_decode(bytes).expect("Failed to decode VerifierJournal[]");
    let wrappers: Vec<VerifierJournalWrapper> = journals
        .into_iter()
        .map(VerifierJournalWrapper::from)
        .collect();
    serde_wasm_bindgen::to_value(&wrappers).expect("Failed to serialize to JsValue")
}

/// Encode a VerifierJournal[] array to ABI-encoded bytes.
///
/// # Arguments
/// * `journals` - A JavaScript array of VerifierJournal objects (see parse_verified_journal for format)
///
/// # Returns
/// ABI-encoded bytes as a Uint8Array
#[wasm_bindgen]
pub fn encode_batch_verified_journal(journals: JsValue) -> Vec<u8> {
    let wrappers: Vec<VerifierJournalWrapper> =
        serde_wasm_bindgen::from_value(journals).expect("Failed to deserialize from JsValue");
    let journals: Vec<VerifierJournal> = wrappers
        .into_iter()
        .map(|w| w.try_into().expect("Failed to convert to VerifierJournal"))
        .collect();
    journals.abi_encode()
}
