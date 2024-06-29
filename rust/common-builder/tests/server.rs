use common_builder::{
    proto::{
        builder_client::BuilderClient, BuildComponentRequest, BuildComponentResponse,
        BundleSourceCodeRequest, BundleSourceCodeResponse, ContentType, Target,
    },
    serve,
};

use tokio::net::TcpListener;

#[tokio::test]
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
            content_type: ContentType::Javascript.into(),
            source_code: format!(
                r#"export * from "http://{}/math/index.js";
"#,
                esm_addr
            ),
        })
        .await?
        .into_inner();

    assert!(bundled_source_code.contains("function add"));

    handler.abort();
    Ok(())
}

#[tokio::test]
async fn it_builds_javascript_modules() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handler = tokio::spawn(async { serve(listener).await.unwrap() });

    let source_code = r#"import { read, write } from 'common:io/state@0.0.1';

export class Body {
    run() {
        const foo = read('foo');
        const value = foo?.deref();

        write('baz', {
          tag: 'string',
          val: 'quux'
        });
    }
}

export const module = {
  Body,

  create() {
      return new Body();
  }
};"#
    .to_owned();

    let mut client = BuilderClient::connect(format!("http://{}", addr)).await?;

    let BuildComponentResponse { id } = client
        .build_component(BuildComponentRequest {
            target: Target::CommonModule.into(),
            content_type: ContentType::Javascript.into(),
            source_code,
        })
        .await?
        .into_inner();

    assert!(!id.is_empty());

    handler.abort();
    Ok(())
}
