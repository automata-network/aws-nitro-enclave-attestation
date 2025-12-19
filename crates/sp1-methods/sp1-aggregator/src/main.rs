#![no_main]
sp1_zkvm::entrypoint!(main);

use aws_nitro_enclave_attestation_verifier::stub::BatchVerifierJournal;
use sp1_zkvm::lib::verify::verify_sp1_proof;

pub fn main() {
    // Read the verification keys.
    let input = sp1_zkvm::io::read_vec();

    let input = BatchVerifierJournal::decode(&input).expect("Failed to decode BatchVerifierJournal");

    let vk_digest: [u32; 8] = unsafe { std::mem::transmute(input.verifierVk) };

    for output in &input.outputs {
        verify_sp1_proof(&vk_digest, &output.digest());
    }

    let journal = BatchVerifierJournal {
        verifierVk: input.verifierVk,
        outputs: input.outputs,
    };

    // Commit the root.
    sp1_zkvm::io::commit_slice(&journal.encode());
}
