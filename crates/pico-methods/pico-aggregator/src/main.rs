#![no_main]
pico_sdk::entrypoint!(main);
use pico_sdk::{
    io::{commit_bytes, read_vec},
    verify::verify_pico_proof,
};

use aws_nitro_enclave_attestation_verifier::stub::{BatchVerifierJournal};

pub fn main() {
    let input_bytes = read_vec();

    let input =
        BatchVerifierJournal::decode(&input_bytes).expect("Failed to decode BatchVerifierJournal");

    let vk_digest: [u32; 8] = unsafe { std::mem::transmute(input.verifierVk) };

    for output in &input.outputs {
        verify_pico_proof(&vk_digest, &output.digest());
    }

    let journal = BatchVerifierJournal {
        verifierVk: input.verifierVk,
        outputs: input.outputs,
    };

    commit_bytes(&journal.encode());
}
