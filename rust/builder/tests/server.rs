use builder::serve;
use reqwest::{
    multipart::{Form, Part},
    Client,
};
use tokio::net::TcpListener;

#[tokio::test]
async fn it_bundles_javascript() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handler = tokio::spawn(async { serve(listener).await.unwrap() });

    let source = format!(
        r#"export * from "https://esm.sh/canvas-confetti@1.6.0";
"#
    );
    let form = Form::new().part("source", Part::text(source).file_name("foo.js"));

    let res = Client::new()
        .post(format!("http://{}/api/v0/bundle", addr.to_string()))
        .multipart(form)
        .send()
        .await?;

    assert_eq!(res.status(), 200);
    assert!(res.text().await?.contains("URL.createObjectURL"));

    handler.abort();
    Ok(())
}

#[tokio::test]
#[ignore]
async fn it_builds_javascript_modules() -> anyhow::Result<()> {
    Ok(())
}
