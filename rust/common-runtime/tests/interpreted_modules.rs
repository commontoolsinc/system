use anyhow::Result;
use common_builder::serve as serve_builder;

use common_protos::{common, runtime};

use common_runtime::serve as serve_runtime;
use common_test_fixtures::sources::common::BASIC_MODULE_JS;
use common_tracing::common_tracing;
use tokio::net::TcpListener;

#[tokio::test(flavor = "multi_thread")]
#[common_tracing]
async fn it_interprets_and_runs_a_common_script() -> Result<()> {
    let builder_listener = TcpListener::bind("127.0.0.1:0").await?;
    let builder_address = builder_listener.local_addr()?;
    let builder_task = tokio::task::spawn(serve_builder(builder_listener));

    std::env::set_var("BUILDER_ADDRESS", format!("http://{}", builder_address));

    let runtime_listener = TcpListener::bind("127.0.0.1:0").await?;
    let runtime_address = runtime_listener.local_addr()?;
    let runtime_task = tokio::task::spawn(serve_runtime(runtime_listener));

    let mut runtime_client =
        runtime::runtime_client::RuntimeClient::connect(format!("http://{}", runtime_address))
            .await?;

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
            module_reference: Some(
                runtime::instantiate_module_request::ModuleReference::ModuleSource(
                    common::ModuleSource {
                        target: common::Target::CommonScript.into(),
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

    builder_task.abort();
    runtime_task.abort();
    Ok(())
}
