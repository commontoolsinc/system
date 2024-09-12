#![cfg(not(target_arch = "wasm32"))]

use common_builder::serve;
use common_protos::{
    builder::{
        builder_client::BuilderClient, BuildComponentRequest, BuildComponentResponse,
        BundleSourceCodeRequest, BundleSourceCodeResponse,
    },
    common::{ContentType, ModuleSource, SourceCode, Target},
};
use common_test_fixtures::sources::common::BASIC_MODULE_JS;
use common_tracing::*;
use tokio::net::TcpListener;

#[tokio::test]
#[common_tracing]
async fn it_bundles_javascript() -> anyhow::Result<()> {
    let mut esm_server = common_test_fixtures::EsmTestServer::default();
    let esm_addr = esm_server.start().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;

    let addr = listener.local_addr()?;
    let handler = tokio::spawn(async { serve(listener).await.unwrap() });

    let mut client = BuilderClient::connect(format!("http://{}", addr)).await?;

    let BundleSourceCodeResponse {
        bundled_source_code,
    } = client
        .bundle_source_code(BundleSourceCodeRequest {
            target: Target::CommonFunction.into(),
            module_source: Some(ModuleSource {
                source_code: [(
                    "module".to_owned(),
                    SourceCode {
                        content_type: ContentType::JavaScript.into(),
                        body: format!(
                            r#"export * from "http://{}/math/index.js";
    "#,
                            esm_addr
                        )
                        .into(),
                    },
                )]
                .into(),
            }),
        })
        .await?
        .into_inner();

    assert!(bundled_source_code.contains("function add"));

    handler.abort();
    Ok(())
}

#[tokio::test]
#[common_tracing]
async fn it_builds_javascript_modules() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handler = tokio::spawn(async { serve(listener).await.unwrap() });

    let source_code = Vec::from(BASIC_MODULE_JS);

    let mut client = BuilderClient::connect(format!("http://{}", addr)).await?;

    let BuildComponentResponse { id } = client
        .build_component(BuildComponentRequest {
            target: Target::CommonFunction.into(),
            module_source: Some(ModuleSource {
                source_code: [(
                    "module".to_owned(),
                    SourceCode {
                        content_type: ContentType::JavaScript.into(),
                        body: source_code,
                    },
                )]
                .into(),
            }),
        })
        .await?
        .into_inner();

    assert!(!id.is_empty());

    handler.abort();
    Ok(())
}
