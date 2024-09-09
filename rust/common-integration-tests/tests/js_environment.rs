#![cfg(not(target_arch = "wasm32"))]

use anyhow::Result;
use common_protos::{
    common,
    runtime::{self, runtime_client::RuntimeClient},
};
use common_runtime::{
    helpers::{start_runtime, VirtualEnvironment},
    Value,
};
use common_test_fixtures::sources::common::{GET_GLOBAL_THIS_PROPS, GET_IMPORT_META_PROPS};
use common_tracing::common_tracing;
use serde_json::json;

#[tokio::test]
#[common_tracing]
async fn it_has_an_empty_import_meta() -> Result<()> {
    // NOTE: The expected values are downstream from Deno's bundler, which
    // shadows `import.meta` keys for compatibility reasons
    let expected = "[\"main\",\"undefined\",\"url\",\"bundler:root\"]";
    let VirtualEnvironment {
        mut runtime_client, ..
    } = start_runtime().await?;
    let output = exec_module(&mut runtime_client, GET_IMPORT_META_PROPS).await?;
    assert_eq!(output, Some(Value::String(expected.to_string())));

    Ok(())
}

#[tokio::test]
#[common_tracing]
async fn it_has_expected_globals() -> Result<()> {
    // Should remove:
    // * Atomics
    // * SharedArrayBuffer
    // * TypedArray (seems like an implementation detail)
    let expected = json!([
        "AggregateError",
        "Array",
        "ArrayBuffer",
        "Atomics",
        "BigInt",
        "BigInt64Array",
        "BigUint64Array",
        "Boolean",
        "DataView",
        "Date",
        "Error",
        "EvalError",
        "Float32Array",
        "Float64Array",
        "Function",
        "Infinity",
        "Int16Array",
        "Int32Array",
        "Int8Array",
        "JSON",
        "Map",
        "Math",
        "NaN",
        "Number",
        "Object",
        "Promise",
        "Proxy",
        "RangeError",
        "Reference",
        "ReferenceError",
        "Reflect",
        "RegExp",
        "Set",
        "SharedArrayBuffer",
        "String",
        "Symbol",
        "SyntaxError",
        "TypeError",
        "TypedArray",
        "URIError",
        "Uint16Array",
        "Uint32Array",
        "Uint8Array",
        "Uint8ClampedArray",
        "WeakMap",
        "WeakRef",
        "WeakSet",
        "console",
        "decodeURI",
        "decodeURIComponent",
        "encodeURI",
        "encodeURIComponent",
        "eval",
        "globalThis",
        "isFinite",
        "isNaN",
        "parseFloat",
        "parseInt",
        "undefined"
    ]);

    let VirtualEnvironment {
        mut runtime_client, ..
    } = start_runtime().await?;

    let output = exec_module(&mut runtime_client, GET_GLOBAL_THIS_PROPS).await?;
    assert_eq!(output, Some(Value::String(expected.to_string())));

    Ok(())
}

/// Instantiates and runs a module with a single input "input" and
/// returns the value of a single output "output".
async fn exec_module(
    runtime_client: &mut RuntimeClient<tonic::transport::channel::Channel>,
    module_str: &str,
) -> Result<Option<Value>> {
    let runtime::InstantiateModuleResponse { instance_id, .. } = runtime_client
        .instantiate_module(runtime::InstantiateModuleRequest {
            output_shape: [("output".into(), common::ValueKind::String.into())].into(),
            default_input: [(
                "input".into(),
                common::Value {
                    variant: Some(common::value::Variant::String("initial foo".into())),
                },
            )]
            .into(),
            target: common::Target::CommonFunctionVm.into(),
            module_reference: Some(common::ModuleBody {
                variant: Some(common::module_body::Variant::ModuleSource(
                    common::ModuleSource {
                        source_code: [(
                            "module".into(),
                            common::SourceCode {
                                content_type: common::ContentType::JavaScript.into(),
                                body: module_str.into(),
                            },
                        )]
                        .into(),
                    },
                )),
            }),
        })
        .await?
        .into_inner();

    let runtime::RunModuleResponse { output } = runtime_client
        .run_module(runtime::RunModuleRequest {
            instance_id,
            input: [(
                "input".into(),
                common::LabeledData {
                    value: Some(common::Value {
                        variant: Some(common::value::Variant::String("updated foo".into())),
                    }),
                    confidentiality: "Public".into(),
                    integrity: "LowIntegrity".into(),
                },
            )]
            .into(),
        })
        .await?
        .into_inner();

    match output.get("output").cloned() {
        Some(data) => match data.value {
            Some(value) => Ok(Some(value.try_into()?)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}
