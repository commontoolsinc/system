use std::fs::copy;
use std::{path::PathBuf, process::Command};

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let project_root_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    // If the environment variable is not set, set it to the path of the generated WASM file
    if std::env::var("COMMON_JAVASCRIPT_INTERPRETER_WASM_PATH").is_err() {
        println!(
            "cargo:rustc-env=COMMON_JAVASCRIPT_INTERPRETER_WASM_PATH={}/common_javascript_interpreter.wasm",
            out_dir.display()
        );
    }

    let cached_artifact_file = project_root_dir
        .join(".wasm_cache")
        .join("common_javascript_interpreter.wasm");

    if cached_artifact_file.is_file() {
        println!("cargo::rerun-if-changed={}", cached_artifact_file.display());

        copy(
            cached_artifact_file,
            out_dir.join("common_javascript_interpreter.wasm"),
        )
        .unwrap();
    } else {
        println!(
            "cargo::rerun-if-changed={}/rust/common-javascript-interpreter",
            project_root_dir.display()
        );
        println!(
            "cargo::rerun-if-changed={}/build/components",
            project_root_dir.display()
        );

        if !Command::new("./scripts/generate-artifacts.sh")
            .status()
            .unwrap()
            .success()
        {
            println!("Failed to generate required artifacts");
        }
    }
}
