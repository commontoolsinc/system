use ct_runtime::{Instance, Module, ModuleDefinition, Result, Runtime, VirtualMachine};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::wasm_bindgen_test;
#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_dedicated_worker);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn it_runs_a_js_vm() -> Result<()> {
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
