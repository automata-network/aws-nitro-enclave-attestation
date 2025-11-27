use risc0_zkvm::compute_image_id;

pub const RISC0_VERIFIER_ELF: &[u8] = include_bytes!("../elf/risc0-verifier-elf");
pub const RISC0_AGGREGATOR_ELF: &[u8] = include_bytes!("../elf/risc0-aggregator-elf");

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
