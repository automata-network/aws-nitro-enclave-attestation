/// Whether the SP1 ELFs embedded in this binary were built reproducibly
/// (i.e., inside docker or sourced from a CI-verified prebuilt). Reflects
/// the `REPRODUCIBLE_BUILD` env var as observed by `sp1-methods/build.rs`
/// at compile time and exposed via the build-helper crate's
/// `ZKVM_REPRODUCIBLE_BUILD` rustc-env token.
pub const SP1_REPRODUCIBLE_BUILD: bool =
    matches!(env!("ZKVM_REPRODUCIBLE_BUILD").as_bytes(), b"1");

use lazy_static::lazy_static;
use sp1_sdk::{Elf, blocking::{EnvProver, EnvProvingKey, Prover}, include_elf};

pub const SP1_VERIFIER_ELF: Elf = include_elf!("sp1-verifier");
pub const SP1_AGGREGATOR_ELF: Elf = include_elf!("sp1-aggregator");

lazy_static! {
    pub static ref ENV_PROVER: EnvProver = EnvProver::new();
    pub static ref SP1_VERIFIER_PK: EnvProvingKey = pk(SP1_VERIFIER_ELF);
    pub static ref SP1_AGGREGATOR_PK: EnvProvingKey = pk(SP1_AGGREGATOR_ELF);
}

fn pk(elf: Elf) -> EnvProvingKey {
    let pk = ENV_PROVER.setup(elf).unwrap();
    pk
}
