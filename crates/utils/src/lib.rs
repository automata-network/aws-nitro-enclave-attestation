//! AWS Nitro Enclave Attestation Utils
//!
//! Utility crate providing WASM bindings for frontend integration.
//! Specifically designed for parsing and encoding VerifierJournal and
//! BatchVerifierJournal data structures.

mod wrapper;

#[cfg(feature = "wasm")]
mod wasm;

// Re-export core types from verifier crate
pub use aws_nitro_enclave_attestation_verifier::stub::{
    BatchVerifierJournal, Bytes48, Pcr, VerificationResult, VerifierInput, VerifierJournal,
    ZkCoProcessorType,
};

// Re-export wrapper types (always available for library use)
pub use wrapper::{PcrWrapper, VerifierJournalWrapper};

#[cfg(feature = "wasm")]
pub use wasm::{
    encode_batch_verified_journal, encode_verified_journal, parse_batch_verified_journal,
    parse_verified_journal,
};
