use std::{env, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("builder_descriptor.bin"))
        .compile(&["proto/builder/builder.proto"], &["proto"])
        .unwrap();

    println!("cargo:rerun-if-changed=./proto/builder/builder.proto");
}
