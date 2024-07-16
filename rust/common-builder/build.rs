use std::path::PathBuf;

const SHIM_COMPONENTS: &[&str] = &[
    "clocks",
    "filesystem",
    "http",
    "io",
    "random",
    "sockets",
    "cli",
];

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let project_root_dir = manifest_dir.parent().unwrap().parent().unwrap();
    let wasi_shim_dir = project_root_dir
        .join("typescript/node_modules/@bytecodealliance/preview2-shim/lib/browser/");

    for component in SHIM_COMPONENTS {
        let env_var = format!("COMMON_WASI_SHIM_{}", component.to_uppercase());
        let file_path = wasi_shim_dir.join(format!("{}.js", component));

        println!("cargo::rustc-env={}={}", env_var, file_path.display());
        println!("cargo::rerun-if-changed={}", file_path.display());
    }

    // TODO: May have to build node_modules here
}
