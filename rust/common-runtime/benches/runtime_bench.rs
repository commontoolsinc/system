use anyhow::Result;
use common_builder::{serve as serve_builder, BuilderError};
use common_protos::{
    builder::{builder_client::BuilderClient, BuildComponentRequest, BuildComponentResponse},
    common::{
        self as common, ContentType, ModuleSignature, ModuleSource, SourceCode, Target, Value,
    },
    runtime::{
        instantiate_module_request::ModuleReference, runtime_client::RuntimeClient,
        InstantiateModuleRequest, InstantiateModuleResponse, InstantiationMode, RunModuleRequest,
        RunModuleResponse,
    },
};
use common_runtime::{serve as serve_runtime, CommonRuntimeError};
use common_test_fixtures::sources::common::BASIC_MODULE_JS;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use tokio::{net::TcpListener, task::JoinHandle};
use tonic::transport::Channel;

async fn init_servers() -> Result<(
    RuntimeClient<Channel>,
    BuilderClient<Channel>,
    JoinHandle<Result<(), CommonRuntimeError>>,
    JoinHandle<Result<(), BuilderError>>,
)> {
    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_address = builder_listener.local_addr()?;
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));
    let builder_client = BuilderClient::connect(format!("http://{}", builder_address)).await?;

    std::env::set_var("BUILDER_ADDRESS", format!("http://{}", builder_address));
    let runtime_listener = TcpListener::bind("127.0.0.1:0").await?;
    let runtime_address = runtime_listener.local_addr()?;
    let runtime_task = tokio::task::spawn(serve_runtime(runtime_listener));

    let runtime_client = RuntimeClient::connect(format!("http://{}", runtime_address)).await?;

    Ok((runtime_client, builder_client, runtime_task, builder_task))
}

async fn build(builder_client: &mut BuilderClient<Channel>) -> Result<String> {
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
    Ok(module_id)
}

async fn instantiate(
    runtime_client: &mut RuntimeClient<Channel>,
    module_id: String,
) -> Result<String> {
    let InstantiateModuleResponse { instance_id, .. } = runtime_client
        .instantiate_module(InstantiateModuleRequest {
            mode: InstantiationMode::Compile.into(),
            output_shape: [("bar".into(), common::ValueKind::String.into())].into(),
            default_input: [(
                "foo".into(),
                Value {
                    variant: Some(common::value::Variant::String("initial foo".into())),
                },
            )]
            .into(),
            module_reference: Some(ModuleReference::ModuleSignature(ModuleSignature {
                target: Target::CommonModule.into(),
                id: module_id,
            })),
        })
        .await?
        .into_inner();
    Ok(instance_id)
}

async fn run(
    mut runtime_client: RuntimeClient<Channel>,
    instance_id: String,
) -> Result<HashMap<String, Value>> {
    let RunModuleResponse { output } = runtime_client
        .run_module(RunModuleRequest {
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
    Ok(output)
}

fn run_benchmark(c: &mut Criterion) {
    let async_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let (instance_id, runtime_client, _builder_client, _runtime_task, _builder_task) =
        async_runtime.block_on(async {
            let (mut runtime_client, mut builder_client, runtime_task, builder_task) =
                init_servers().await.unwrap();
            let module_id = build(&mut builder_client).await.unwrap();
            let instance_id = instantiate(&mut runtime_client, module_id).await.unwrap();
            (
                instance_id,
                runtime_client,
                builder_client,
                runtime_task,
                builder_task,
            )
        });

    let input = 0;
    c.bench_with_input(
        BenchmarkId::new("Runtime::run_module", &input),
        &input,
        move |b, _| {
            b.to_async(&async_runtime)
                .iter(|| run(runtime_client.clone(), instance_id.clone()));
        },
    );
}

criterion_group!(benches, run_benchmark);
criterion_main!(benches);
