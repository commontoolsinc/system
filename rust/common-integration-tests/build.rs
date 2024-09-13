fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    if let ("wasm32", "unknown", Some(_)) = (target_arch.as_str(), target_os.as_str(), std::option_env!("COMMON_BROWSER_INTEGRATION_TEST")) {
        println!("cargo::rerun-if-env-changed=COMMON_RUNTIME_PORT");
        println!("cargo::rerun-if-env-changed=COMMON_BUILDER_PORT");
        println!("cargo:rustc-cfg=common_browser_integration_test");
    };
}
