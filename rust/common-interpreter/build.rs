use std::process::Command;

const TYPESCRIPT_SOURCE_DEPENDENCIES: &[&str] = &[
    "common/io/wit/io.wit",
    "common/data/wit/data.wit",
    "common/module/wit/module.wit",
];

fn main() {
    if !Command::new("wit-deps").status().unwrap().success() {
        panic!("Failed to run wit-deps");
    }

    for fragment in TYPESCRIPT_SOURCE_DEPENDENCIES.iter() {
        println!("cargo:rerun-if-changed=../../typescript/{fragment}");
    }
}
