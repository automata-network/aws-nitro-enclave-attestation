use risc0_build::{embed_methods_with_options, DockerOptionsBuilder, GuestOptionsBuilder};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const RISC0_VERIFIER: &str = "risc0-verifier";
const RISC0_AGGREGATOR: &str = "risc0-aggregator";

fn main() {
    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let elf_dir = manifest_dir.join("elf");

    // Check if pre-built ELFs exist
    let verifier_path = elf_dir.join(format!("{}-elf", RISC0_VERIFIER));
    let aggregator_path = elf_dir.join(format!("{}-elf", RISC0_AGGREGATOR));

    if verifier_path.exists() && aggregator_path.exists() {
        println!("cargo::warning=Skipping build (pre-built ELFs exist in ./elf/)");
        return;
    }

    // Build
    let use_docker = std::env::var("USE_DOCKER").is_ok();

    let mut builder = GuestOptionsBuilder::default();
    if use_docker {
        let docker_options = DockerOptionsBuilder::default()
            .root_dir(manifest_dir.join("../../"))
            .build()
            .unwrap();
        builder.use_docker(docker_options);
    }

    let guest_options = builder.build().unwrap();
    embed_methods_with_options(HashMap::from([
        (RISC0_VERIFIER, guest_options.clone()),
        (RISC0_AGGREGATOR, guest_options),
    ]));

    // Copy ELFs to ./elf/ for future builds
    fs::create_dir_all(&elf_dir).unwrap();

    let profile = if use_docker { "docker" } else { "release" };

    // Use CARGO_TARGET_DIR if set, otherwise use workspace root's target/
    let target_base = manifest_dir.join("../../target");

    for name in [RISC0_VERIFIER, RISC0_AGGREGATOR] {
        // Path: target/riscv-guest/risc0-methods/{elf-name}/riscv32im-risc0-zkvm-elf/{profile}/{elf-name}
        let src = target_base.join(format!(
            "riscv-guest/risc0-methods/{}/riscv32im-risc0-zkvm-elf/{}/{}",
            name, profile, name
        ));
        let dst = elf_dir.join(format!("{}-elf", name));
        println!("cargo::warning=Copying {:?} to {:?}", src, dst);
        fs::copy(&src, &dst).unwrap_or_else(|e| panic!("Failed to copy {:?} to {:?}: {}", src, dst, e));
    }
}
