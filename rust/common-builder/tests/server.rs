use common_builder::serve;
use reqwest::{
    multipart::{Form, Part},
    Client,
};
use tokio::net::TcpListener;

#[tokio::test]
async fn it_bundles_javascript() -> anyhow::Result<()> {
    let mut esm_server = common_test_fixtures::EsmTestServer::default();
    let esm_addr = esm_server.start().await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handler = tokio::spawn(async { serve(listener).await.unwrap() });

    let source = format!(
        r#"export * from "http://{}/math/index.js";
"#,
        esm_addr
    );
    let form = Form::new()
        .part("source", Part::text(source).file_name("foo.js"))
        .part("target", Part::text("common:module"));

    let res = Client::new()
        .post(format!("http://{}/api/v0/bundle", addr))
        .multipart(form)
        .send()
        .await?;

    assert_eq!(res.status(), 200);
    assert!(res.text().await?.contains("function add"));

    handler.abort();
    Ok(())
}

#[tokio::test]
async fn it_builds_javascript_modules() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handler = tokio::spawn(async { serve(listener).await.unwrap() });

    let source = r#"import { read, write } from 'common:io/state@0.0.1';

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
};"#;
    let form = Form::new()
        .part("source", Part::text(source).file_name("foo.js"))
        .part("target", Part::text("common:module"));

    let res = Client::new()
        .post(format!("http://{}/api/v0/module", addr))
        .multipart(form)
        .send()
        .await?;

    assert_eq!(res.status(), 200);
    assert!(res.text().await?.starts_with("{\"id\":"));

    handler.abort();
    Ok(())
}
