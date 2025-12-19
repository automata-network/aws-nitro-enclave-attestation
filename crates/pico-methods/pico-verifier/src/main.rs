#![no_main]
pico_sdk::entrypoint!(main);
use pico_sdk::io::{commit_bytes, read_vec};

use aws_nitro_enclave_attestation_verifier::{verify_attestation_report, stub::VerifierInput};

pub fn main() {
    let input_bytes = read_vec();
    let input = VerifierInput::decode(&input_bytes).unwrap();

    let output = verify_attestation_report(&input).unwrap();

    commit_bytes(&output.encode());
}