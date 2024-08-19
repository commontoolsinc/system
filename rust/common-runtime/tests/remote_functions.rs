#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use common_ifc::{Confidentiality, Integrity, ModuleEnvironment, Policy};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use common_runtime::{
    remote::function::WebRemoteFunctionContext, Affinity, BasicIo, ContentType, FunctionInterface,
    HasModuleContext, IoData, IoShape, IoValues, ModuleBody, ModuleContext, ModuleDefinition,
    ModuleDriver, ModuleFactory, RemoteFunctionDefinition, SourceCode, SourceCodeCollection,
    Validated, Value, ValueKind, WebRuntime,
};
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use common_wit::Target;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use anyhow::Result;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use http::Uri;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use std::{collections::BTreeMap, str::FromStr};

use common_macros::common_browser_integration_test;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_service_worker);

#[common_browser_integration_test]
async fn it_runs_a_remote_common_function_vm() -> Result<()> {
    let runtime_port = std::option_env!("COMMON_RUNTIME_PORT").unwrap();
    let runtime = WebRuntime::new(Some(Uri::from_str(&format!(
        "http://127.0.0.1:{runtime_port}"
    ))?))?;

    let definition = RemoteFunctionDefinition::try_from(ModuleDefinition {
        target: Target::CommonFunctionVm,
        affinity: Affinity::RemoteOnly,
        inputs: IoShape::from(BTreeMap::from([("foo".to_string(), ValueKind::String)])),
        outputs: IoShape::from(BTreeMap::from([("bar".to_string(), ValueKind::String)])),
        body: ModuleBody::SourceCode(SourceCodeCollection::from([(
            "main".to_string(),
            SourceCode {
                content_type: ContentType::JavaScript,
                body: common_test_fixtures::sources::common::BASIC_MODULE_JS.into(),
            },
        )])),
    })?;

    let factory = runtime.prepare(definition).await?;

    let default_input = IoValues::from(BTreeMap::from([(
        "foo".into(),
        Value::String("initial foo".into()),
    )]));
    let output_shape = IoShape::from(BTreeMap::from([("bar".into(), ValueKind::String)]));
    let initial_io = BasicIo::from_initial_state(default_input, output_shape.clone());

    let mut instance = factory
        .instantiate(WebRemoteFunctionContext::new(
            initial_io,
            common_ifc::Context {
                environment: ModuleEnvironment::Server,
            },
        ))
        .await?;

    let run_input = {
        let input = IoData::from(BTreeMap::from([(
            "foo".into(),
            (
                Value::from("updated foo"),
                Confidentiality::Public,
                Integrity::Low,
            )
                .into(),
        )]));

        BasicIo::new(input, output_shape)
    };
    let policy = Policy::with_defaults()?;
    let validated_run_input =
        Validated::try_from((&policy, instance.context().ifc(), run_input.clone())).unwrap();

    let result = instance.run(validated_run_input).await?;

    assert_eq!(
        result.get("bar").unwrap().value,
        Value::String("updated foo:bar".into())
    );

    Ok(())
}

#[common_browser_integration_test]
async fn it_runs_a_remote_common_function() -> Result<()> {
    let runtime_port = std::option_env!("COMMON_RUNTIME_PORT").unwrap();
    let runtime = WebRuntime::new(Some(Uri::from_str(&format!(
        "http://127.0.0.1:{runtime_port}"
    ))?))?;

    let definition = RemoteFunctionDefinition::try_from(ModuleDefinition {
        target: Target::CommonFunction,
        affinity: Affinity::RemoteOnly,
        inputs: IoShape::from(BTreeMap::from([("foo".to_string(), ValueKind::String)])),
        outputs: IoShape::from(BTreeMap::from([("bar".to_string(), ValueKind::String)])),
        body: ModuleBody::SourceCode(SourceCodeCollection::from([(
            "main".to_string(),
            SourceCode {
                content_type: ContentType::JavaScript,
                body: common_test_fixtures::sources::common::BASIC_MODULE_JS.into(),
            },
        )])),
    })?;

    let factory = runtime.prepare(definition).await?;

    let default_input = IoValues::from(BTreeMap::from([(
        "foo".into(),
        Value::String("initial foo".into()),
    )]));
    let output_shape = IoShape::from(BTreeMap::from([("bar".into(), ValueKind::String)]));
    let initial_io = BasicIo::from_initial_state(default_input, output_shape.clone());

    let mut instance = factory
        .instantiate(WebRemoteFunctionContext::new(
            initial_io,
            common_ifc::Context {
                environment: ModuleEnvironment::Server,
            },
        ))
        .await?;

    let run_input = {
        let input = IoData::from(BTreeMap::from([(
            "foo".into(),
            (
                Value::from("updated foo"),
                Confidentiality::Public,
                Integrity::Low,
            )
                .into(),
        )]));

        BasicIo::new(input, output_shape)
    };
    let policy = Policy::with_defaults()?;
    let validated_run_input =
        Validated::try_from((&policy, instance.context().ifc(), run_input.clone())).unwrap();

    let result = instance.run(validated_run_input).await?;

    assert_eq!(
        result.get("bar").unwrap().value,
        Value::String("updated foo:bar".into())
    );

    Ok(())
}
