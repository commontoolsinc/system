#![cfg(not(target_arch = "wasm32"))]

use ct_builder::{serve, BuildServerConfig, ImportMap};
use ct_tracing::ct_tracing;
use std::path::PathBuf;
use tokio::net::TcpListener;

const RECIPE_GEN: &'static str = r#"
import { add } from "test:math";

export const run = () => {
  return {
    "fakemodule": add(5, 8),
  }
}
"#;

#[tokio::test]
#[ct_tracing]
async fn it_builds_and_returns_recipe_over_http() -> anyhow::Result<()> {
    let import_map_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/imports.json");
    let import_map = ImportMap::from_path(&import_map_path).await?;

    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let config = BuildServerConfig::default()
        .with_http(listener)
        .with_import_map(import_map);
    let _handler = tokio::spawn(async { serve(config).await.unwrap() });
    let client = reqwest::Client::new();
    let recipe = client
        .post(format!("http://{}/recipe", addr))
        .body(RECIPE_GEN)
        .send()
        .await?
        .text()
        .await?;
    assert_eq!(recipe, "{\"fakemodule\":13}");
    Ok(())
}
