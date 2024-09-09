#![cfg(not(target_arch = "wasm32"))]

use anyhow::Result;
use common_protos::{common, runtime};
use common_runtime::helpers::{start_runtime, VirtualEnvironment};
use common_test_fixtures::sources::common::BASIC_MODULE_JS;
use common_tracing::common_tracing;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[common_tracing]
async fn it_interprets_and_runs_a_common_script() -> Result<()> {
    let VirtualEnvironment {
        mut runtime_client, ..
    } = start_runtime().await?;

    let runtime::InstantiateModuleResponse { instance_id, .. } = runtime_client
        .instantiate_module(runtime::InstantiateModuleRequest {
            // TODO: This needs to be removed, instead use the `target` associated with the module source
            output_shape: [("bar".into(), common::ValueKind::String.into())].into(),
            default_input: [(
                "foo".into(),
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
                                body: BASIC_MODULE_JS.into(),
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
                "foo".into(),
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

    assert_eq!(
        output.get("bar"),
        Some(&common::LabeledData {
            value: Some(common::Value {
                variant: Some(common::value::Variant::String("updated foo:bar".into()))
            }),
            confidentiality: "Public".into(),
            integrity: "LowIntegrity".into(),
        })
    );
    Ok(())
}
