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
        .file_descriptor_set_path(out_dir.join("builder_descriptor.bin"))
        .compile(&["builder/builder.proto"], &[proto_path.clone()])
        .unwrap();

    println!(
        "cargo:rerun-if-changed={}/builder/builder.proto",
        proto_path.display()
    );
}
