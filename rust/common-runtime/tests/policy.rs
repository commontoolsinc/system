#![cfg(not(target_arch = "wasm32"))]

use anyhow::Result;
use common_builder::{serve as serve_builder, BuilderError};
use common_ifc::{
    Confidentiality, Context as IfcContext, Data, Integrity, ModuleEnvironment, Policy,
};
use common_runtime::{
    target::function::{NativeFunctionContext, NativeFunctionFactory},
    Affinity, ArtifactResolver, BasicIo, ContentType, FunctionDefinition, FunctionInterface,
    HasModuleContext, InputOutput, IoData, IoShape, ModuleBody, ModuleContext, ModuleDefinition,
    ModuleDriver, ModuleFactory, NativeRuntime, SourceCode, Validated, Value, ValueKind,
};
use common_test_fixtures::sources::common::BASIC_MODULE_JS;
use common_tracing::common_tracing;
use common_wit::Target;
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

async fn get_basic_js_module(runtime: &NativeRuntime) -> Result<(NativeFunctionFactory, IoShape)> {
    let input_shape = IoShape::from(BTreeMap::from([("foo".into(), ValueKind::String)]));
    let output_shape = IoShape::from(BTreeMap::from([("bar".into(), ValueKind::String)]));

    let factory = runtime
        .prepare(FunctionDefinition::try_from(ModuleDefinition {
            target: Target::CommonFunction,
            affinity: Affinity::LocalOnly,
            inputs: input_shape,
            outputs: output_shape,
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
        })?)
        .await?;

    Ok((
        factory,
        IoShape::from(BTreeMap::from([("bar".into(), ValueKind::String)])),
    ))
}

#[macro_export]
macro_rules! assert_io {
    ( $instance:expr, $policy:expr, { $($in_key:expr => $in_val:expr),* } , { $($out_key:expr => $out_val:expr),* }) => {
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

        let input_io = BasicIo::new(input, $instance.context().io().output_shape().clone());
        let validated_input_io = Validated::try_from(($policy, $instance.context().ifc(), input_io))?;

        let output = $instance.run(validated_input_io).await?;
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

    let artifact_resolver = ArtifactResolver::new(Some(builder_address))?;
    let runtime = NativeRuntime::new(artifact_resolver)?;

    let (factory, output_shape) = get_basic_js_module(&runtime).await?;

    let mut instance = factory
        .instantiate(NativeFunctionContext::new(
            BasicIo::new(IoData::default(), output_shape),
            IfcContext {
                environment: ModuleEnvironment::Server,
            },
        ))
        .await?;

    // Private/HighIntegrity stays that way
    assert_io!(
        &mut instance,
        &Policy::with_defaults()?,
        {
            "foo" => ("foo", Confidentiality::Private, Integrity::High)
        },
        {
           "bar" => ("foo:bar", Confidentiality::Private, Integrity::High)
        }
    );

    // Public/LowIntegrity stays that way
    assert_io!(
        &mut instance,
        &Policy::with_defaults()?,
        {
            "foo" => ("foo", Confidentiality::Public, Integrity::Low)
        },
        {
           "bar" => ("foo:bar", Confidentiality::Public, Integrity::Low)
        }
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[common_tracing]
async fn it_rejects_based_on_env() -> Result<()> {
    let (builder_address, _) = init_build_server().await?;
    let artifact_resolver = ArtifactResolver::new(Some(builder_address))?;
    let runtime = NativeRuntime::new(artifact_resolver)?;

    let (factory, output_shape) = get_basic_js_module(&runtime).await?;

    let instance = factory
        .instantiate(NativeFunctionContext::new(
            BasicIo::new(IoData::default(), output_shape),
            IfcContext {
                environment: ModuleEnvironment::Server,
            },
        ))
        .await?;

    // Private data only on BrowserClient
    let policy = Policy::new(
        [
            (Confidentiality::Public, (ModuleEnvironment::Server,).into()),
            (
                Confidentiality::Private,
                (ModuleEnvironment::WebBrowser,).into(),
            ),
        ],
        [
            (Integrity::Low, (ModuleEnvironment::Server,).into()),
            (Integrity::High, (ModuleEnvironment::Server,).into()),
        ],
    )?;

    let input = IoData::from(BTreeMap::from([(
        "foo".into(),
        Data::from((
            Value::from("foo"),
            Confidentiality::Private,
            Integrity::High,
        )),
    )]));

    let input_io = BasicIo::new(input, IoShape::from(instance.context().io().input()));

    assert!(Validated::try_from((policy, instance.context().ifc(), input_io)).is_err());

    Ok(())
}
