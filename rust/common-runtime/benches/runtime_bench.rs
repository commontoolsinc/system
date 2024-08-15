#![cfg(not(target_arch = "wasm32"))]

use anyhow::Result;
use common_builder::{serve as serve_builder, BuilderError};
use common_ifc::{Confidentiality, Integrity, Policy};
use common_runtime::{
    target::{
        function::{NativeFunction, NativeFunctionContext},
        function_vm::{NativeFunctionVm, NativeFunctionVmContext},
    },
    Affinity, ArtifactResolver, BasicIo, CommonRuntimeError, ContentType, FunctionDefinition,
    FunctionInterface, FunctionVmDefinition, HasModuleContext, IoData, IoShape, IoValues,
    ModuleBody, ModuleContext, ModuleDefinition, ModuleDriver, ModuleFactory, NativeRuntime,
    SourceCode, Validated, Value, ValueKind,
};
use common_test_fixtures::sources::common::BASIC_MODULE_JS;
use common_wit::Target;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use http::Uri;
use std::{collections::BTreeMap, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex, task::JoinHandle};

/// Start a build server, returning its address and a task handler.
async fn init_build_server() -> Result<(Uri, JoinHandle<Result<(), BuilderError>>)> {
    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_url = format!("http://{}", builder_listener.local_addr()?);
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));

    Ok((builder_url.parse()?, builder_task))
}

fn make_basic_js_definition(target: Target) -> ModuleDefinition {
    let inputs = IoShape::from(BTreeMap::from([("foo".into(), ValueKind::String)]));
    let outputs = IoShape::from(BTreeMap::from([("bar".into(), ValueKind::String)]));

    ModuleDefinition {
        target,
        affinity: Affinity::LocalOnly,
        inputs,
        outputs,
        body: ModuleBody::SourceCode(
            [(
                "module".to_owned(),
                SourceCode {
                    content_type: ContentType::JavaScript,
                    body: BASIC_MODULE_JS.into(),
                },
            )]
            .into(),
        ),
    }
}

fn run_benchmark(criterion: &mut Criterion) {
    let async_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let (_runtime, function, function_vm, output_shape) = async_runtime
        .block_on(async {
            let (builder_address, _builder_task) = init_build_server().await.unwrap();
            let artifact_resolver = ArtifactResolver::new(Some(builder_address))?;
            let runtime = NativeRuntime::new(artifact_resolver)?;

            let default_input = IoValues::from(BTreeMap::from([(
                "foo".into(),
                Value::String("initial foo".into()),
            )]));
            let output_shape = IoShape::from(BTreeMap::from([("bar".into(), ValueKind::String)]));

            let io = BasicIo::from_initial_state(default_input, output_shape.clone());

            let function_factory = runtime
                .prepare(FunctionDefinition::try_from(make_basic_js_definition(
                    Target::CommonFunction,
                ))?)
                .await?;
            let function = function_factory
                .instantiate(NativeFunctionContext::new(
                    io.clone(),
                    common_ifc::Context {
                        environment: common_ifc::ModuleEnvironment::Server,
                    },
                ))
                .await?;

            let function_vm_factory = runtime
                .prepare(FunctionVmDefinition::try_from(make_basic_js_definition(
                    Target::CommonFunctionVm,
                ))?)
                .await?;
            let function_vm = function_vm_factory
                .instantiate(NativeFunctionVmContext::new(
                    io,
                    common_ifc::Context {
                        environment: common_ifc::ModuleEnvironment::Server,
                    },
                ))
                .await?;

            Ok((
                runtime,
                Arc::new(Mutex::new(function)),
                Arc::new(Mutex::new(function_vm)),
                output_shape,
            ))
                as Result<
                    (
                        NativeRuntime,
                        Arc<Mutex<NativeFunction>>,
                        Arc<Mutex<NativeFunctionVm>>,
                        IoShape,
                    ),
                    CommonRuntimeError,
                >
        })
        .unwrap();

    let bench_input = {
        let input = IoData::from(BTreeMap::from([(
            "foo".into(),
            (
                Value::from("updated foo"),
                Confidentiality::Low,
                Integrity::Low,
            )
                .into(),
        )]));

        BasicIo::new(input, output_shape)
    };
    let policy = Policy::with_defaults().unwrap();

    let mut group = criterion.benchmark_group("run_benchmark");

    group.bench_with_input(
        BenchmarkId::new("function", ""),
        &bench_input.clone(),
        |bencher, io| {
            bencher.to_async(&async_runtime).iter(|| async {
                let mut function = function.lock().await;
                let validated_input =
                    Validated::try_from((&policy, function.context().ifc(), io.clone())).unwrap();
                function.run(validated_input).await
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("function_vm", ""),
        &bench_input.clone(),
        |bencher, io| {
            bencher.to_async(&async_runtime).iter(|| async {
                let mut function_vm = function_vm.lock().await;
                let validated_input =
                    Validated::try_from((&policy, function_vm.context().ifc(), io.clone()))
                        .unwrap();
                function_vm.run(validated_input).await
            })
        },
    );
}

criterion_group!(benches, run_benchmark);
criterion_main!(benches);
