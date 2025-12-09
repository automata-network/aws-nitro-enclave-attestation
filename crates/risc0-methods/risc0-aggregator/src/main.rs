use aws_nitro_enclave_attestation_verifier::stub::{BatchVerifierInput, BatchVerifierJournal};
use risc0_zkvm::{guest::env, Receipt};
use serde::Deserialize;

/// Input format for the aggregator when used with Boundless.
/// The host sends a postcard-serialized tuple of:
/// - ABI-encoded BatchVerifierInput bytes
/// - Vector of receipts (one per verification)
#[derive(Deserialize)]
struct AggregatorInput {
    input_bytes: Vec<u8>,
    receipts: Vec<Receipt>,
}

fn main() {
    // Read postcard-serialized input: (ABI-encoded BatchVerifierInput, Vec<Receipt>)
    let frame_bytes = env::read_frame();
    let aggregator_input: AggregatorInput =
        postcard::from_bytes(&frame_bytes).expect("Failed to decode postcard input");

    let input = BatchVerifierInput::decode(&aggregator_input.input_bytes)
        .expect("Failed to decode BatchVerifierInput");

    // Verify each receipt matches the expected output
    assert_eq!(
        aggregator_input.receipts.len(),
        input.outputs.len(),
        "Number of receipts must match number of outputs"
    );

    for (receipt, expected_output) in aggregator_input.receipts.iter().zip(input.outputs.iter()) {
        // Verify the receipt against the verifier image ID
        receipt
            .verify(input.verifierVk.0)
            .expect("Failed to verify receipt");

        // Verify journal matches expected output
        assert_eq!(
            receipt.journal.bytes,
            expected_output.encode(),
            "Receipt journal does not match expected output"
        );
    }

    // Reuse input.outputs directly since we verified the receipts
    let journal = BatchVerifierJournal {
        verifierVk: input.verifierVk,
        outputs: input.outputs,
    };

    // write public output to the journal
    env::commit_slice(&journal.encode());
}
