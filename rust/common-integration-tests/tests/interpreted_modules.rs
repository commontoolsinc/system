#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;

use anyhow::Result;
use common_protos::{
    common::{self, LabeledData},
    runtime::{self, runtime_client::RuntimeClient},
};
use common_runtime::helpers::{start_runtime, VirtualEnvironment};
use common_tracing::common_tracing;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[common_tracing]
async fn it_interprets_and_runs_a_common_script() -> Result<()> {
    let VirtualEnvironment {
        mut runtime_client, ..
    } = start_runtime().await?;

    let runtime::InstantiateModuleResponse { instance_id, .. } = runtime_client
        .instantiate_module(runtime::InstantiateModuleRequest {
            output_shape: [
                ("str-out".into(), common::ValueKind::String.into()),
                ("num-out".into(), common::ValueKind::Number.into()),
                ("bool-out".into(), common::ValueKind::Boolean.into()),
            ]
            .into(),
            default_input: [
                (
                    "str".into(),
                    common::Value {
                        variant: Some(common::value::Variant::String("initial foo".into())),
                    },
                ),
                (
                    "num".into(),
                    common::Value {
                        variant: Some(common::value::Variant::Number(0.0)),
                    },
                ),
                (
                    "bool".into(),
                    common::Value {
                        variant: Some(common::value::Variant::Boolean(false)),
                    },
                ),
            ]
            .into(),
            target: common::Target::CommonFunctionVm.into(),
            module_reference: Some(common::ModuleBody {
                variant: Some(common::module_body::Variant::ModuleSource(
                    common::ModuleSource {
                        source_code: [(
                            "module".into(),
                            common::SourceCode {
                                content_type: common::ContentType::JavaScript.into(),
                                body: r#"import { read, write } from "common:io/state@0.0.1";
export const run = () => {
  const str = read("str")?.deref()?.val;
  const num = read("num")?.deref()?.val;
  const bool = read("bool")?.deref()?.val;

  write("str-out", {
    tag: "string",
    val: `${str}:new`,
  });
  write("num-out", {
    tag: "number",
    val: num + 1,
  });
  write("bool-out", {
    tag: "boolean",
    val: !bool,
  });
};
"#
                                .into(),
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
                input: [
                    (
                        "str".into(),
                        common::LabeledData {
                            value: Some(common::Value {
                                variant: Some(common::value::Variant::String("foo".into())),
                            }),
                            confidentiality: "Public".into(),
                            integrity: "LowIntegrity".into(),
                        },
                    ),
                    (
                        "num".into(),
                        common::LabeledData {
                            value: Some(common::Value {
                                variant: Some(common::value::Variant::Number(10.0.into())),
                            }),
                            confidentiality: "Public".into(),
                            integrity: "LowIntegrity".into(),
                        },
                    ),
                    (
                        "bool".into(),
                        common::LabeledData {
                            value: Some(common::Value {
                                variant: Some(common::value::Variant::Boolean(false)),
                            }),
                            confidentiality: "Public".into(),
                            integrity: "LowIntegrity".into(),
                        },
                    ),
                ]
                .into(),
            })
            .await?
            .into_inner();
        Ok(output)
    }

    for keep_alive in [true, false] {
        let output = run(&mut runtime_client, &instance_id, keep_alive).await?;
        assert_eq!(
            output.get("str-out"),
            Some(&common::LabeledData {
                value: Some(common::Value {
                    variant: Some(common::value::Variant::String("foo:new".into()))
                }),
                confidentiality: "Public".into(),
                integrity: "LowIntegrity".into(),
            })
        );
        assert_eq!(
            output.get("num-out"),
            Some(&common::LabeledData {
                value: Some(common::Value {
                    variant: Some(common::value::Variant::Number(11.0)),
                }),
                confidentiality: "Public".into(),
                integrity: "LowIntegrity".into(),
            })
        );
        assert_eq!(
            output.get("bool-out"),
            Some(&common::LabeledData {
                value: Some(common::Value {
                    variant: Some(common::value::Variant::Boolean(true)),
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
