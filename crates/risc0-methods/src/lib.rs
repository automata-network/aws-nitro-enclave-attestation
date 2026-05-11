/// Whether the RISC0 ELFs embedded in this binary were built reproducibly
/// (i.e., inside docker or sourced from a CI-verified prebuilt). Reflects
/// the `REPRODUCIBLE_BUILD` env var as observed by `risc0-methods/build.rs`
/// at compile time and exposed via the build-helper crate's
/// `ZKVM_REPRODUCIBLE_BUILD` rustc-env token.
pub const RISC0_REPRODUCIBLE_BUILD: bool =
    matches!(env!("ZKVM_REPRODUCIBLE_BUILD").as_bytes(), b"1");

pub const RISC0_VERIFIER_ELF: &[u8] =
    include_bytes!(env!("RISC0_ELF_risc0-verifier"));
pub const RISC0_AGGREGATOR_ELF: &[u8] =
    include_bytes!(env!("RISC0_ELF_risc0-aggregator"));

use risc0_zkvm::compute_image_id;

lazy_static::lazy_static! {
    pub static ref RISC0_VERIFIER_ID: [u32; 8] = compute_image_id(RISC0_VERIFIER_ELF)
        .expect("Failed to compute RISC0_VERIFIER_ID")
        .as_words()
        .try_into()
        .expect("Image ID should be 8 words");
    pub static ref RISC0_AGGREGATOR_ID: [u32; 8] = compute_image_id(RISC0_AGGREGATOR_ELF)
        .expect("Failed to compute RISC0_AGGREGATOR_ID")
        .as_words()
        .try_into()
        .expect("Image ID should be 8 words");
}
