use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let proto_path = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("proto");

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("runtime_descriptor.bin"))
        .compile(
            &[
                "common/common.proto",
                "builder/builder.proto",
                "runtime/runtime.proto",
            ],
            &[proto_path.clone()],
        )
        .unwrap();

    for path in [
        "/common/common.proto",
        "/builder/builder.proto",
        "/runtime/runtime.proto",
    ] {
        println!("cargo:rerun-if-changed={}{}", proto_path.display(), path);
    }
}
