use sp1_build::{build_program_with_args, BuildArgs};

fn main() {
    build_program_with_args("../sp1-methods/sp1-verifier", get_build_args());
    build_program_with_args("../sp1-methods/sp1-aggregator", get_build_args());
}

fn get_build_args() -> BuildArgs {
    let use_docker = std::env::var("USE_DOCKER").is_ok();
    BuildArgs {
        docker: use_docker,
        ..Default::default()
    }
}