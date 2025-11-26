use sp1_build::{build_program_with_args, BuildArgs};

fn main() {
    build_program_with_args("../sp1-methods/sp1-verifier", get_build_args());
    build_program_with_args("../sp1-methods/sp1-aggregator", get_build_args());
}

fn get_build_args() -> BuildArgs {
    let use_docker = std::env::var("USE_DOCKER").is_ok();

    // Get the workspace root (2 directories up from crates/sp1-methods)
    // This is needed for Docker builds since guest programs have their own [workspace]
    // marker for deterministic Cargo.lock, but we need Docker to mount the full project
    // so path dependencies (e.g., ../../verifier) are accessible.
    let workspace_directory = if use_docker {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR not set");
        let workspace_root = std::path::Path::new(&manifest_dir)
            .parent() // -> crates/
            .and_then(|p| p.parent()) // -> project root
            .expect("Failed to find workspace root");
        Some(workspace_root.to_string_lossy().to_string())
    } else {
        None
    };

    BuildArgs {
        docker: use_docker,
        workspace_directory,
        ..Default::default()
    }
}
