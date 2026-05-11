use std::path::{Path, PathBuf};
use sp1_build::BuildArgs;
use zkvm_program_build::{ProgramBuild, ZkVmBackend};

struct Sp1Backend;

impl ZkVmBackend for Sp1Backend {
    fn env_prefix() -> &'static str {
        "SP1"
    }

    fn build_one(
        target: &str,
        ws_root: &Path,
        docker: bool,
        output_dir: Option<&Path>,
    ) -> PathBuf {
        let mut args = BuildArgs {
            docker,
            workspace_directory: Some(ws_root.to_string_lossy().into_owned()),
            ..Default::default()
        };
        let produced = if let Some(dir) = output_dir {
            args.output_directory = Some(dir.to_string_lossy().into_owned());
            args.elf_name = Some(format!("{target}.elf"));
            dir.join(format!("{target}.elf"))
        } else {
            // Local mode: sp1-build writes to its default target/ location AND
            // emits `cargo:rustc-env=SP1_ELF_<target>=<path>` itself from inside
            // `build_program_with_args`. Return an empty path so the default
            // `wire()` does not re-emit (which would clobber the value).
            PathBuf::new()
        };
        sp1_build::build_program_with_args(target, args);
        produced
    }

    // wire(): uses default trait impl. Emits SP1_ELF_<target>=<path> for
    // Prebuilt and Docker (non-empty path); skips Local (empty path) so
    // sp1-build's internal emission wins.
}

fn main() {
    ProgramBuild::<Sp1Backend>::from_env()
        .build_all(&["sp1-verifier", "sp1-aggregator"]);
}
