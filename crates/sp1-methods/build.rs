use sp1_build::{build_program_with_args, BuildArgs};
use std::path::Path;

const SP1_VERIFIER: &str = "sp1-verifier";
const SP1_AGGREGATOR: &str = "sp1-aggregator";

fn main() {
    build_if_missing(SP1_VERIFIER);
    build_if_missing(SP1_AGGREGATOR);
}

fn build_if_missing(program_name: &str) {
    let elf_path = format!("./elf/{}-elf", program_name);

    if Path::new(&elf_path).exists() {
        println!(
            "cargo::warning=Skipping build for {} (ELF exists at {})",
            program_name, elf_path
        );
        return;
    }

    build_program_with_args(
        format!("../sp1-methods/{}", program_name).as_str(),
        get_build_args(program_name),
    );
}

fn get_build_args(program_name: &str) -> BuildArgs {
    let use_docker = std::env::var("USE_DOCKER").is_ok();

    // Get the workspace root (2 directories up from crates/sp1-methods)
    // This is needed for Docker builds since guest programs have their own [workspace]
    // marker for deterministic Cargo.lock, but we need Docker to mount the full project
    // so path dependencies (e.g., ../../verifier) are accessible.
    let workspace_directory = if use_docker {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let workspace_root = std::path::Path::new(&manifest_dir)
            .parent() // -> crates/
            .and_then(|p| p.parent()) // -> project root
            .expect("Failed to find workspace root");
        Some(workspace_root.to_string_lossy().to_string())
    } else {
        None
    };

    let output_directory = Some("./elf".to_string());
    let elf_name = Some(format!("{}-elf", program_name));

    BuildArgs {
        docker: use_docker,
        workspace_directory,
        output_directory,
        elf_name,
        ..Default::default()
    }
}
