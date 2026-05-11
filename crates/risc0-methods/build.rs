use std::fs;
use std::path::{Path, PathBuf};
use risc0_build::{
    build_package, get_package, get_target_dir, DockerOptionsBuilder, GuestOptionsBuilder,
};
use zkvm_program_build::{ProgramBuild, ZkVmBackend};

struct Risc0Backend;

impl ZkVmBackend for Risc0Backend {
    fn env_prefix() -> &'static str {
        "RISC0"
    }

    fn build_one(
        target: &str,
        ws_root: &Path,
        docker: bool,
        output_dir: Option<&Path>,
    ) -> PathBuf {
        let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
        let pkg_manifest_dir = manifest_dir.join(target);
        let pkg = get_package(&pkg_manifest_dir);

        // Mirror risc0-build's internal target-dir layout:
        //   <target>/riscv-guest/<host-crate>/<guest-crate>
        let target_base = get_target_dir(pkg_manifest_dir.join("Cargo.toml"));
        let guest_target_dir = target_base
            .join("riscv-guest")
            .join("risc0-methods")
            .join(target);

        let mut opts_builder = GuestOptionsBuilder::default();
        if docker {
            let docker_opts = DockerOptionsBuilder::default()
                .root_dir(ws_root.to_path_buf())
                .build()
                .unwrap();
            opts_builder.use_docker(docker_opts);
        }
        let opts = opts_builder.build().unwrap();

        let entries = build_package(&pkg, &guest_target_dir, opts)
            .unwrap_or_else(|e| panic!("risc0 build_package({target}) failed: {e}"));
        let entry = entries
            .into_iter()
            .next()
            .unwrap_or_else(|| panic!("risc0 build_package({target}) returned no entries"));
        let produced: PathBuf = entry.path.as_ref().into();

        if let Some(dir) = output_dir {
            fs::create_dir_all(dir).unwrap();
            let dst = dir.join(format!("{target}.elf"));
            fs::copy(&produced, &dst).unwrap_or_else(|e| {
                panic!("Failed to copy {produced:?} to {dst:?}: {e}")
            });
            dst
        } else {
            produced
        }
    }

    // wire(): uses default trait impl. RISC0's `build_one` always returns a
    // real path (no SDK-internal env emission), so the default emits
    // `cargo:rustc-env=RISC0_ELF_<target>=<path>` for every target/mode.
}

fn main() {
    ProgramBuild::<Risc0Backend>::from_env()
        .build_all(&["risc0-verifier", "risc0-aggregator"]);
}
