#![cfg(not(target_arch = "wasm32"))]

use anyhow::Result;
use common_builder::{serve as serve_builder, BuilderError};
use common_runtime::{
    ContentType, ModuleSource, RawModule, Runtime, RuntimeIo, SourceCode, Value, ValueKind,
};
use common_test_fixtures::sources::common::BASIC_MODULE_JS;
use common_wit::Target;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use http::Uri;
use std::collections::BTreeMap;
use tokio::{net::TcpListener, task::JoinHandle};

/// Start a build server, returning its address and a task handler.
async fn init_build_server() -> Result<(Uri, JoinHandle<Result<(), BuilderError>>)> {
    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_url = format!("http://{}", builder_listener.local_addr()?);
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));

    Ok((builder_url.parse()?, builder_task))
}

/// Wrapper for module prefabs for benchmarks, containing
/// module source, default input, and output shape.
struct BenchModule {
    module_source: ModuleSource,
    output_shape: BTreeMap<String, ValueKind>,
    default_input: BTreeMap<String, Value>,
}

impl BenchModule {
    /// Turns this [BenchModule] into a [RawModule] and [RuntimeIo], the
    /// needed components to compile this module.
    pub fn into_components(self, builder_address: Option<Uri>) -> (RawModule, RuntimeIo) {
        let module = RawModule::new(self.module_source, builder_address);
        let initial_io = RuntimeIo::new(self.default_input, self.output_shape);
        (module, initial_io)
    }

    /// Module definition for [BASIC_MODULE_JS].
    pub fn new_basic_js_module() -> Self {
        Self {
            module_source: ModuleSource {
                target: Target::CommonModule.into(),
                source_code: [(
                    "module".to_owned(),
                    SourceCode {
                        content_type: ContentType::JavaScript.into(),
                        body: BASIC_MODULE_JS.into(),
                    },
                )]
                .into(),
            },
            output_shape: [("bar".into(), ValueKind::String)].into(),
            default_input: [("foo".into(), Value::String("initial foo".into()))].into(),
        }
    }
}

fn run_benchmark(c: &mut Criterion) {
    let async_runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let (runtime, instance_id, _builder_task) = async_runtime.block_on(async {
        let (builder_address, builder_task) = init_build_server().await.unwrap();
        let mut runtime = Runtime::new().unwrap();
        let (module, initial_io) =
            BenchModule::new_basic_js_module().into_components(Some(builder_address.clone()));
        let instance_id = runtime.compile(module, initial_io).await.unwrap();

        (runtime, instance_id, builder_task)
    });

    let bench_input = {
        let input = [("foo".into(), Value::String("updated foo".into()))].into();
        let output_shape = runtime.output_shape(&instance_id).unwrap().to_owned();
        RuntimeIo::new(input, output_shape)
    };

    c.bench_with_input(
        BenchmarkId::new("Runtime::run_module", ""),
        &bench_input,
        |b, runtime_io| {
            b.to_async(&async_runtime).iter(|| {
                // As RuntimeIo is owned, containing dynamic BTreeMaps,
                // we can't avoid either construction or cloning, responsible
                // for ~70us
                runtime.run(&instance_id, runtime_io.to_owned())
            })
        },
    );
}

criterion_group!(benches, run_benchmark);
criterion_main!(benches);
