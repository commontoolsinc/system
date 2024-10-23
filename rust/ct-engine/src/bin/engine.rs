#[cfg(not(target_arch = "wasm32"))]
#[macro_use]
extern crate tracing;

#[cfg(target_arch = "wasm32")]
pub fn main() {
    unimplemented!("Binary not supported for wasm32")
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
pub async fn main() -> ct_runtime::Result<()> {
    use ct_runtime::{Instance, Module, ModuleDefinition, Runtime, VirtualMachine};

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

    let runtime = Runtime::new()?;
    let mut module = runtime.module(definition).await?;
    let mut instance = module.instantiate().await?;

    let input = r#"{"foo":9}"#;
    let output = instance.run(input.into()).await?;
    assert_eq!(output, r#"{"foo":10}"#);

    Ok(())
}
