#[cfg(target_arch = "wasm32")]
pub fn main() {
    unimplemented!("Binary not supported for wasm32")
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() -> ct_runtime::Result<()> {
    use ct_runtime::{HostFeatures, Instance, Module, ModuleDefinition, Runtime, VirtualMachine};

    println!("Running test case in lieu of a service.");

    let source = r#"
    export const run = (input) => {
      input.foo = input.foo + 1;
      return input;
    }
    "#;
    let definition = ModuleDefinition {
        vm: VirtualMachine::JavaScript,
        source: source.into(),
    };

    struct Host;
    impl HostFeatures for Host {
        fn host_callback(input: String) -> std::result::Result<String, String> {
            Ok(input)
        }
    }
    let runtime = Runtime::<Host>::new()?;
    let mut module = runtime.module(definition)?;
    let mut instance = module.instantiate()?;

    let input = r#"{"foo":9}"#;
    let output = instance.run(input.into())?;
    assert_eq!(output, r#"{"foo":10}"#);

    Ok(())
}
