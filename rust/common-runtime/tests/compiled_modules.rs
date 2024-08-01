#![cfg(not(target_arch = "wasm32"))]

mod shared;

use anyhow::Result;
use common_builder::serve as serve_builder;
use common_protos::{builder, common, runtime};
use common_runtime::serve as serve_runtime;
use common_test_fixtures::sources::common::BASIC_MODULE_JS;
use common_tracing::common_tracing;
use shared::start_runtime;
use tokio::net::TcpListener;

#[tokio::test]
#[common_tracing]
async fn it_compiles_and_runs_an_uncompiled_module() -> Result<()> {
    let (mut runtime_client, _, _) = start_runtime().await?;

    let runtime::InstantiateModuleResponse { instance_id, .. } = runtime_client
        .instantiate_module(runtime::InstantiateModuleRequest {
            output_shape: [("bar".into(), common::ValueKind::String.into())].into(),
            default_input: [(
                "foo".into(),
                common::Value {
                    variant: Some(common::value::Variant::String("initial foo".into())),
                },
            )]
            .into(),
            module_reference: Some(
                runtime::instantiate_module_request::ModuleReference::ModuleSource(
                    common::ModuleSource {
                        target: common::Target::CommonFunction.into(),
                        source_code: [(
                            "module".into(),
                            common::SourceCode {
                                content_type: common::ContentType::JavaScript.into(),
                                body: BASIC_MODULE_JS.into(),
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
                "foo".into(),
                common::Value {
                    variant: Some(common::value::Variant::String("updated foo".into())),
                },
            )]
            .into(),
        })
        .await?
        .into_inner();

    assert_eq!(
        output.get("bar"),
        Some(&common::Value {
            variant: Some(common::value::Variant::String("updated foo:bar".into()))
        })
    );
    Ok(())
}

#[tokio::test]
#[common_tracing]
async fn it_runs_a_precompiled_module() -> Result<()> {
    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_address_str = format!("http://{}", builder_listener.local_addr()?);
    let builder_address: http::Uri = builder_address_str.parse()?;
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));

    let mut builder_client =
        builder::builder_client::BuilderClient::connect(builder_address_str).await?;

    let builder::BuildComponentResponse { id: module_id } = builder_client
        .build_component(builder::BuildComponentRequest {
            module_source: Some(common::ModuleSource {
                target: common::Target::CommonFunction.into(),
                source_code: [(
                    "module".to_owned(),
                    common::SourceCode {
                        content_type: common::ContentType::JavaScript.into(),
                        body: BASIC_MODULE_JS.into(),
                    },
                )]
                .into(),
            }),
        })
        .await?
        .into_inner();

    let runtime_listener = TcpListener::bind("127.0.0.1:0").await?;
    let runtime_address = runtime_listener.local_addr()?;
    let runtime_task = tokio::task::spawn(serve_runtime(runtime_listener, Some(builder_address)));

    let mut runtime_client =
        runtime::runtime_client::RuntimeClient::connect(format!("http://{}", runtime_address))
            .await?;

    let runtime::InstantiateModuleResponse { instance_id, .. } = runtime_client
        .instantiate_module(runtime::InstantiateModuleRequest {
            output_shape: [("bar".into(), common::ValueKind::String.into())].into(),
            default_input: [(
                "foo".into(),
                common::Value {
                    variant: Some(common::value::Variant::String("initial foo".into())),
                },
            )]
            .into(),
            module_reference: Some(
                runtime::instantiate_module_request::ModuleReference::ModuleSignature(
                    common::ModuleSignature {
                        target: common::Target::CommonFunction.into(),
                        id: module_id,
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
                "foo".into(),
                common::Value {
                    variant: Some(common::value::Variant::String("updated foo".into())),
                },
            )]
            .into(),
        })
        .await?
        .into_inner();

    assert_eq!(
        output.get("bar"),
        Some(&common::Value {
            variant: Some(common::value::Variant::String("updated foo:bar".into()))
        })
    );

    builder_task.abort();
    runtime_task.abort();
    Ok(())
}
