use std::marker::PhantomData;
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BuildMode {
    Prebuilt,
    Docker,
    Local,
}

/// Parse the `REPRODUCIBLE_BUILD` env value into a `BuildMode`.
///
/// Pure function — kept separate from `from_env` so it is unit-testable.
/// Comparison is case-insensitive after trimming surrounding whitespace.
pub fn parse_mode(raw: Option<&str>) -> BuildMode {
    let v = raw.map(|s| s.trim().to_ascii_lowercase());
    match v.as_deref() {
        None | Some("") | Some("prebuilt") => BuildMode::Prebuilt,
        Some("docker") => BuildMode::Docker,
        _ => BuildMode::Local,
    }
}

/// Per-zkVM hook surface. Implementations live in each methods crate's `build.rs`
/// as a unit struct (e.g. `Sp1Backend`, `Risc0Backend`).
pub trait ZkVmBackend {
    /// Prefix used for this zkVM's ELF env vars. The default `wire()` emits
    /// `cargo:rustc-env=<PREFIX>_ELF_<target>=<path>` per target, and the
    /// consumer crate's `src/lib.rs` reads the same name via
    /// `include_bytes!(env!("<PREFIX>_ELF_<target>"))`.
    ///
    /// Examples: `"SP1"`, `"RISC0"`.
    fn env_prefix() -> &'static str;

    /// File name for the prebuilt ELF in `<crate>/elf/`. Default `<target>.elf`.
    fn prebuilt_file_name(target: &str) -> String {
        format!("{target}.elf")
    }

    /// Run the zkVM-specific build for one `target`.
    ///
    /// - `ws_root` is the cargo workspace root.
    /// - `docker = true` means reproducible (containerized) build.
    /// - `output_dir = Some(elf_dir)` in Docker mode, `None` in Local mode.
    ///   When `Some`, the impl MUST write to
    ///   `output_dir/<prebuilt_file_name(target)>`. When `None`, the impl
    ///   may leave the ELF wherever the SDK puts it (typically `target/...`).
    ///
    /// Returns the absolute path to the produced ELF. If the backend's SDK
    /// already emits `cargo:rustc-env=<PREFIX>_ELF_<target>=<path>` internally
    /// (e.g. SP1's `build_program_with_args` in Local mode), return
    /// `PathBuf::new()` so the default `wire()` skips re-emission for that
    /// target.
    fn build_one(
        target: &str,
        ws_root: &Path,
        docker: bool,
        output_dir: Option<&Path>,
    ) -> PathBuf;

    /// Default: emit `cargo:rustc-env=<env_prefix>_ELF_<target>=<path>` and
    /// `cargo:rerun-if-changed=<path>` for each non-empty path. Override
    /// only if a backend needs an entirely different wiring mechanism.
    fn wire(targets: &[(&str, PathBuf)]) {
        for (target, path) in targets {
            if path.as_os_str().is_empty() {
                continue;
            }
            println!(
                "cargo:rustc-env={}_ELF_{}={}",
                Self::env_prefix(),
                target,
                path.display()
            );
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}

/// Build-mode orchestrator parameterized by a `ZkVmBackend`. Read via
/// `from_env()` once at the start of `build.rs`, then call `build_all(&[...])`
/// with the target names.
pub struct ProgramBuild<B: ZkVmBackend> {
    mode: BuildMode,
    _marker: PhantomData<B>,
}

impl<B: ZkVmBackend> ProgramBuild<B> {
    pub fn from_env() -> Self {
        println!("cargo:rerun-if-env-changed=REPRODUCIBLE_BUILD");
        let mode = parse_mode(std::env::var("REPRODUCIBLE_BUILD").ok().as_deref());
        Self { mode, _marker: PhantomData }
    }

    pub fn mode(&self) -> BuildMode {
        self.mode
    }

    pub fn reproducible(&self) -> bool {
        matches!(self.mode, BuildMode::Prebuilt | BuildMode::Docker)
    }

    pub fn build_all(&self, targets: &[&str]) {
        let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
        let elf_dir = manifest_dir.join("elf");
        let ws_root = manifest_dir
            .parent()
            .and_then(Path::parent)
            .expect("Failed to find workspace root")
            .to_path_buf();

        let mut wired: Vec<(&str, PathBuf)> = Vec::with_capacity(targets.len());
        for &target in targets {
            let path = match self.mode {
                BuildMode::Prebuilt => {
                    let p = elf_dir.join(B::prebuilt_file_name(target));
                    if !p.exists() {
                        panic!(
                            "REPRODUCIBLE_BUILD=prebuilt but {} is missing.\n\
                             Hint: REPRODUCIBLE_BUILD=docker cargo build to (re)build, \
                             or REPRODUCIBLE_BUILD=disable for a fast local non-reproducible build.",
                            p.display()
                        );
                    }
                    p
                }
                BuildMode::Docker => B::build_one(target, &ws_root, true, Some(&elf_dir)),
                BuildMode::Local => B::build_one(target, &ws_root, false, None),
            };
            wired.push((target, path));
        }

        B::wire(&wired);

        // Same name for every zkVM; cargo scopes rustc-env per crate, so each
        // consumer crate's `env!("ZKVM_REPRODUCIBLE_BUILD")` reads only the
        // value emitted by its own build.rs.
        println!(
            "cargo:rustc-env=ZKVM_REPRODUCIBLE_BUILD={}",
            if self.reproducible() { "1" } else { "0" }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_mode_unset_is_prebuilt() {
        assert_eq!(parse_mode(None), BuildMode::Prebuilt);
    }

    #[test]
    fn parse_mode_empty_string_is_prebuilt() {
        assert_eq!(parse_mode(Some("")), BuildMode::Prebuilt);
    }

    #[test]
    fn parse_mode_prebuilt_literal_with_whitespace_and_casing() {
        assert_eq!(parse_mode(Some("prebuilt")), BuildMode::Prebuilt);
        assert_eq!(parse_mode(Some("Prebuilt")), BuildMode::Prebuilt);
        assert_eq!(parse_mode(Some("  PREBUILT  ")), BuildMode::Prebuilt);
    }

    #[test]
    fn parse_mode_docker_case_insensitive() {
        assert_eq!(parse_mode(Some("docker")), BuildMode::Docker);
        assert_eq!(parse_mode(Some("DOCKER")), BuildMode::Docker);
        assert_eq!(parse_mode(Some(" Docker ")), BuildMode::Docker);
    }

    #[test]
    fn parse_mode_anything_else_is_local() {
        for raw in ["disable", "0", "false", "no", "off", "garbage", "1", "true"] {
            assert_eq!(parse_mode(Some(raw)), BuildMode::Local, "raw={raw}");
        }
    }
}
