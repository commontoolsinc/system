const WIT_DEPENDENCIES: &[&str] = &[
    "common/io/wit/io.wit",
    "common/data/wit/data.wit",
    "common/function/wit/function.wit",
];

fn main() {
    for fragment in WIT_DEPENDENCIES.iter() {
        println!("cargo:rerun-if-changed=../../wit/{fragment}");
    }
}
