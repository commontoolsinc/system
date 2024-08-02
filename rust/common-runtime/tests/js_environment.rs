#![cfg(not(target_arch = "wasm32"))]

mod shared;
use anyhow::Result;
use common_protos::{
    common,
    runtime::{self, runtime_client::RuntimeClient},
};
use common_test_fixtures::sources::common::{GET_GLOBAL_THIS_PROPS, GET_IMPORT_META_PROPS};
use common_tracing::common_tracing;
use serde_json::json;
use shared::start_runtime;

#[tokio::test]
#[common_tracing]
async fn it_has_an_empty_import_meta() -> Result<()> {
    let expected = "[]";
    let (mut runtime_client, _, _) = start_runtime().await?;
    let output = exec_module(&mut runtime_client, GET_IMPORT_META_PROPS).await?;
    assert_eq!(
        output,
        Some(common::Value {
            variant: Some(common::value::Variant::String(expected.to_string().into())),
        }),
    );

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

    let (mut runtime_client, _, _) = start_runtime().await?;
    let output = exec_module(&mut runtime_client, GET_GLOBAL_THIS_PROPS).await?;
    assert_eq!(
        output,
        Some(common::Value {
            variant: Some(common::value::Variant::String(expected.to_string().into())),
        }),
    );

    Ok(())
}

/// Instantiates and runs a module with a single input "input" and
/// returns the value of a single output "output".
async fn exec_module(
    runtime_client: &mut RuntimeClient<tonic::transport::channel::Channel>,
    module_str: &str,
) -> Result<Option<common::Value>> {
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
            module_reference: Some(
                runtime::instantiate_module_request::ModuleReference::ModuleSource(
                    common::ModuleSource {
                        target: common::Target::CommonFunctionVm.into(),
                        source_code: [(
                            "module".into(),
                            common::SourceCode {
                                content_type: common::ContentType::JavaScript.into(),
                                body: module_str.into(),
                            },
                        )]
                        .into(),
                    },
                ),
            ),
        })
        .await?
        .into_inner();

    let runtime::RunModuleResponse { output } = runtime_client
        .run_module(runtime::RunModuleRequest {
            instance_id,
            input: [(
                "input".into(),
                common::Value {
                    variant: Some(common::value::Variant::String("updated foo".into())),
                },
            )]
            .into(),
        })
        .await?
        .into_inner();

    Ok(output.get("output").cloned())
}
