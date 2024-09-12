#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;

use anyhow::Result;
use common_protos::{
    common::{self, LabeledData},
    runtime::{self, runtime_client::RuntimeClient},
};
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

    async fn run(
        runtime_client: &mut RuntimeClient<tonic::transport::channel::Channel>,
        instance_id: &str,
        keep_alive: bool,
    ) -> Result<HashMap<String, LabeledData>> {
        let runtime::RunModuleResponse { output } = runtime_client
            .run_module(runtime::RunModuleRequest {
                instance_id: instance_id.to_string(),
                keep_alive,
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
        Ok(output)
    }

    for keep_alive in [true, false] {
        let output = run(&mut runtime_client, &instance_id, keep_alive).await?;
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
    }
    // The second run does not keep the module alive.
    // This third run should fail.
    assert!(run(&mut runtime_client, &instance_id, true).await.is_err());
    Ok(())
}
