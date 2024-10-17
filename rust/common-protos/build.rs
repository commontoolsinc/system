use std::{env, path::PathBuf};

const COMMON_SOURCE: &str = "common/common.proto";
const BUILDER_SOURCE: &str = "builder/builder.proto";
const RUNTIME_SOURCE: &str = "runtime/runtime.proto";
const FORMULA_SOURCE: &str = "formula/formula.proto";

fn is_set(var: &str) -> bool {
    env::var(var).is_ok()
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_path = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("proto");

    let mut sources = vec![COMMON_SOURCE];

    if is_set("CARGO_FEATURE_RUNTIME") {
        sources.push(BUILDER_SOURCE);
        sources.push(RUNTIME_SOURCE);
        sources.push(FORMULA_SOURCE);
    } else if is_set("CARGO_FEATURE_BUILDER") {
        sources.push(BUILDER_SOURCE);
    }

    let target = env::var("TARGET").unwrap();

    tonic_build::configure()
        .build_transport(target != "wasm32-unknown-unknown")
        .file_descriptor_set_path(out_dir.join("protos_descriptor.bin"))
        // Will always rebuild unless `emit_rerun_if_changed` is false.
        .emit_rerun_if_changed(false)
        .compile_protos(&sources, &[proto_path.clone()])
        .unwrap();

    for path in [
        FORMULA_SOURCE,
        COMMON_SOURCE,
        BUILDER_SOURCE,
        RUNTIME_SOURCE,
    ] {
        println!("cargo:rerun-if-changed={}/{}", proto_path.display(), path);
    }
}
