use std::process::Command;

const TYPESCRIPT_SOURCE_DEPENDENCIES: &[&str] = &[
    "package.json",
    "package-lock.json",
    "common/io/wit/io.wit",
    "common/data/wit/data.wit",
    "common/module/wit/module.wit",
];

fn main() {
    // Clean node_modules directory to avoid file conflicts
    if !Command::new("rm")
        .arg("-rf")
        .arg("../../typescript/node_modules")
        .status()
        .unwrap()
        .success()
    {
        panic!("Failed to clean node_modules directory");
    }

    if !Command::new("npm")
        .arg("ci")
        .current_dir("../../typescript")
        .status()
        .unwrap()
        .success()
    {
        panic!("Failed to run npm install");
    }

    if !Command::new("npm")
        .arg("run")
        .arg("build")
        .current_dir("../../typescript")
        .status()
        .unwrap()
        .success()
    {
        panic!("Failed to run npm build");
    }

    for fragment in TYPESCRIPT_SOURCE_DEPENDENCIES.iter() {
        println!("cargo:rerun-if-changed=../../typescript/{fragment}");
    }
}
