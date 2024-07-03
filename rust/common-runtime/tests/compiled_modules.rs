use anyhow::Result;
use common_builder::serve as serve_builder;

use common_runtime::protos::{
    self,
    builder::{builder_client::BuilderClient, BuildComponentRequest, BuildComponentResponse},
    common::{ContentType, ModuleId, ModuleSource, SourceCode, Target, ValueType},
    runtime::{
        instantiate_module_request::ModuleReference, runtime_client::RuntimeClient,
        InstantiateModuleRequest, InstantiateModuleResponse, InstantiationMode, RunModuleRequest,
        RunModuleResponse,
    },
};
use common_runtime::serve as serve_runtime;
use common_test_fixtures::sources::common::BASIC_MODULE_JS;
use tokio::net::TcpListener;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, EnvFilter, FmtSubscriber};

#[tokio::test]
async fn it_compiles_and_runs_an_uncompiled_module() -> Result<()> {
    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_address = builder_listener.local_addr()?;
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));

    std::env::set_var("BUILDER_ADDRESS", format!("http://{}", builder_address));

    let runtime_listener = TcpListener::bind("127.0.0.1:0").await?;
    let runtime_address = runtime_listener.local_addr()?;
    let runtime_task = tokio::task::spawn(serve_runtime(runtime_listener));

    let mut runtime_client = RuntimeClient::connect(format!("http://{}", runtime_address)).await?;

    let InstantiateModuleResponse { instance_id, .. } = runtime_client
        .instantiate_module(InstantiateModuleRequest {
            mode: InstantiationMode::Compile.into(),
            output_shape: [("bar".into(), ValueType::String.into())].into(),
            default_input: [(
                "foo".into(),
                protos::common::Value {
                    value_type: Some(protos::common::value::ValueType::String(
                        "initial foo".into(),
                    )),
                },
            )]
            .into(),
            module_reference: Some(ModuleReference::ModuleSource(ModuleSource {
                target: Target::CommonModule.into(),
                source_code: [(
                    "module".into(),
                    SourceCode {
                        content_type: ContentType::JavaScript.into(),
                        body: BASIC_MODULE_JS.into(),
                    },
                )]
                .into(),
            })),
        })
        .await?
        .into_inner();

    let RunModuleResponse { output } = runtime_client
        .run_module(RunModuleRequest {
            instance_id,
            input: [(
                "foo".into(),
                protos::common::Value {
                    value_type: Some(protos::common::value::ValueType::String(
                        "updated foo".into(),
                    )),
                },
            )]
            .into(),
        })
        .await?
        .into_inner();

    assert_eq!(
        output.get("bar"),
        Some(&protos::common::Value {
            value_type: Some(protos::common::value::ValueType::String("baz".into()))
        })
    );

    builder_task.abort();
    runtime_task.abort();
    Ok(())
}

#[tokio::test]
async fn it_runs_a_precompiled_module() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber.with(Layer::default().pretty()))
        .expect("Failed to configure tracing");

    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_address = builder_listener.local_addr()?;
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));

    std::env::set_var("BUILDER_ADDRESS", format!("http://{}", builder_address));

    let mut builder_client = BuilderClient::connect(format!("http://{}", builder_address)).await?;

    let BuildComponentResponse { id: module_id } = builder_client
        .build_component(BuildComponentRequest {
            module_source: Some(ModuleSource {
                target: Target::CommonModule.into(),
                source_code: [(
                    "module".to_owned(),
                    SourceCode {
                        content_type: ContentType::JavaScript.into(),
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
    let runtime_task = tokio::task::spawn(serve_runtime(runtime_listener));

    let mut runtime_client = RuntimeClient::connect(format!("http://{}", runtime_address)).await?;

    let InstantiateModuleResponse { instance_id, .. } = runtime_client
        .instantiate_module(InstantiateModuleRequest {
            mode: InstantiationMode::Compile.into(),
            output_shape: [("bar".into(), ValueType::String.into())].into(),
            default_input: [(
                "foo".into(),
                protos::common::Value {
                    value_type: Some(protos::common::value::ValueType::String(
                        "initial foo".into(),
                    )),
                },
            )]
            .into(),
            module_reference: Some(ModuleReference::ModuleId(ModuleId {
                target: Target::CommonModule.into(),
                id: module_id,
            })),
        })
        .await?
        .into_inner();

    let RunModuleResponse { output } = runtime_client
        .run_module(RunModuleRequest {
            instance_id,
            input: [(
                "foo".into(),
                protos::common::Value {
                    value_type: Some(protos::common::value::ValueType::String(
                        "updated foo".into(),
                    )),
                },
            )]
            .into(),
        })
        .await?
        .into_inner();

    assert_eq!(
        output.get("bar"),
        Some(&protos::common::Value {
            value_type: Some(protos::common::value::ValueType::String("baz".into()))
        })
    );

    builder_task.abort();
    runtime_task.abort();
    Ok(())
}
