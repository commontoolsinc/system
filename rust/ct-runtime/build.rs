use std::{
    fs::{copy, create_dir_all},
    path::{Path, PathBuf},
    process::Command,
};

/// Builds `common-formula-javascript-interpreter` as a
/// wasm32-wasip1 component for inclusion via
/// `include_bytes!(env!("CT_JS_VM_WASM_PATH"));`
fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let project_root_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    build_component(project_root_dir, out_dir.as_path(), "ct-js-vm", "CT_JS_VM");
}

fn build_component(project_root_dir: &Path, out_dir: &Path, crate_name: &str, env_name: &str) {
    let artifact_name = crate_name.replace('-', "_");
    let artifact_file_name = format!("virt_{}.wasm", artifact_name);
    let artifact_dir = format!("{}/wasm32-wasip1/release", out_dir.display());
    let artifact_path = format!("{}/{}", artifact_dir, artifact_file_name);

    let cached_artifact_file = project_root_dir
        .join(".wasm_cache")
        .join(&artifact_file_name);

    if cached_artifact_file.is_file() {
        println!("cargo::rerun-if-changed={}", cached_artifact_file.display());
        create_dir_all(&artifact_dir).unwrap();
        copy(cached_artifact_file, artifact_path.clone()).unwrap();
    } else {
        println!("cargo:warning=Rebuilding {} component", crate_name);
        println!(
            "cargo::rerun-if-changed={}/rust/{}",
            project_root_dir.display(),
            crate_name,
        );

        let mut cmd = Command::new("bash");
        cmd.current_dir(project_root_dir)
            .arg("./scripts/component-build")
            .arg(crate_name)
            .arg(out_dir)
            .env_remove("CARGO_ENCODED_RUSTFLAGS");

        let status = cmd.status().unwrap();
        assert!(status.success());
    }

    let const_name = format!("{}_WASM_PATH", env_name.to_ascii_uppercase());
    println!("cargo::rustc-env={}={}", const_name, artifact_path);
}
