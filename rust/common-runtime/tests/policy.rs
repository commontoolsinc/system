#![cfg(not(target_arch = "wasm32"))]

use anyhow::Result;
use common_builder::{serve as serve_builder, BuilderError};
use common_ifc::{Confidentiality, Data, Integrity, ModuleEnvironment, Policy};
use common_runtime::{
    ContentType, InputOutput, IoData, IoShape, ModuleSource, RawModule, Runtime, RuntimeIo,
    SourceCode, Value, ValueKind,
};
use common_test_fixtures::sources::common::BASIC_MODULE_JS;
use common_tracing::common_tracing;
use common_wit::Target;
use http::Uri;
use std::collections::BTreeMap;
use tokio::{net::TcpListener, task::JoinHandle};

use Confidentiality::*;
use Integrity::*;
use ModuleEnvironment::*;

/// Start a build server, returning its address and a task handler.
async fn init_build_server() -> Result<(Uri, JoinHandle<Result<(), BuilderError>>)> {
    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_url = format!("http://{}", builder_listener.local_addr()?);
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));

    Ok((builder_url.parse()?, builder_task))
}

fn get_basic_js_module(builder_address: Uri) -> (RawModule, IoShape) {
    let source = ModuleSource {
        target: Target::CommonFunctionVm.into(),
        source_code: [(
            "module".to_owned(),
            SourceCode {
                content_type: ContentType::JavaScript.into(),
                body: BASIC_MODULE_JS.into(),
            },
        )]
        .into(),
    };
    (
        RawModule::new(source, Some(builder_address)),
        IoShape::from(BTreeMap::from([("bar".into(), ValueKind::String)])),
    )
}

#[macro_export]
macro_rules! assert_io {
    ( $runtime:expr, $instance_id:expr, $policy:expr, { $($in_key:expr => $in_val:expr),* } , { $($out_key:expr => $out_val:expr),* }) => {
        {
        let mut input = IoData::default();
        $({
            let (val, conf_label, int_label) = $in_val;
            input.insert($in_key.into(), (Value::from(val), conf_label, int_label).into());
        })*
        let mut expected_output = IoData::default();
        $({
            let (val, conf_label, int_label) = $out_val;
            expected_output.insert($out_key.into(), (Value::from(val), conf_label, int_label).into());
        })*

        let input_io = RuntimeIo::new(input, $runtime.output_shape($instance_id)?.to_owned());
        let output_io = $runtime.run($instance_id, input_io, $policy).await?;
        let output = output_io.output();
        let mut expected_keys = vec![];

        for (key, value) in expected_output.iter() {
            expected_keys.push(key.to_owned());
            assert_eq!(output.get(key).unwrap(), value);
        }

        for (key, _) in output.iter() {
            assert!(expected_keys.contains(key));
        }
        }
    };
}

#[tokio::test(flavor = "multi_thread")]
#[common_tracing]
async fn it_propagates_labels() -> Result<()> {
    let (builder_address, _) = init_build_server().await?;
    let mut runtime = Runtime::new()?;
    let (module, output_shape) = get_basic_js_module(builder_address);
    let instance_id = runtime
        .interpret(module, RuntimeIo::new(IoData::default(), output_shape))
        .await?;

    // Private/HighIntegrity stays that way
    assert_io!(
        &mut runtime,
        &instance_id,
        &Policy::with_defaults()?,
        {
            "foo" => ("foo", Private, HighIntegrity)
        },
        {
           "bar" => ("foo:bar", Private, HighIntegrity)
        }
    );

    // Public/LowIntegrity stays that way
    assert_io!(
        &mut runtime,
        &instance_id,
        &Policy::with_defaults()?,
        {
            "foo" => ("foo", Public, LowIntegrity)
        },
        {
           "bar" => ("foo:bar", Public, LowIntegrity)
        }
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[common_tracing]
async fn it_rejects_based_on_env() -> Result<()> {
    let (builder_address, _) = init_build_server().await?;
    let mut runtime = Runtime::new()?;
    let (module, output_shape) = get_basic_js_module(builder_address);
    let instance_id = runtime
        .interpret(module, RuntimeIo::new(IoData::default(), output_shape))
        .await?;

    // Private data only on BrowserClient
    let policy = Policy::new(
        [(Public, (Server,).into()), (Private, (WebBrowser,).into())],
        [
            (LowIntegrity, (Server,).into()),
            (HighIntegrity, (Server,).into()),
        ],
    )?;

    let input = IoData::from(BTreeMap::from([(
        "foo".into(),
        Data::from((Value::from("foo"), Private, HighIntegrity)),
    )]));
    let input_io = RuntimeIo::new(input, runtime.output_shape(&instance_id)?.to_owned());
    assert!(runtime.run(&instance_id, input_io, &policy).await.is_err());

    Ok(())
}
