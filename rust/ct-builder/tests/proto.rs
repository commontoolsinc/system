#![cfg(not(target_arch = "wasm32"))]

use ct_builder::{serve, BuildServerConfig};
use ct_common::ModuleDefinition;
use ct_protos::builder::{
    builder_client::BuilderClient, BuildComponentRequest, BuildComponentResponse,
    ReadComponentRequest, ReadComponentResponse,
};
use ct_test_fixtures::sources::common::BASIC_MODULE_TSX;
use ct_tracing::ct_tracing;
use tokio::net::TcpListener;

#[tokio::test]
#[ct_tracing]
async fn it_builds_and_returns_components_over_grpc() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let config = BuildServerConfig::default().with_grpc(listener);
    let _handler = tokio::spawn(async { serve(config).await.unwrap() });

    let mut client = BuilderClient::connect(format!("http://{}", addr)).await?;

    let BuildComponentResponse { component_id } = client
        .build_component(BuildComponentRequest {
            module_definition: Some(ModuleDefinition::from(BASIC_MODULE_TSX).try_into()?),
            bundle_common_imports: false,
        })
        .await?
        .into_inner();

    let ReadComponentResponse {
        component,
        source_map,
    } = client
        .read_component(ReadComponentRequest { component_id })
        .await?
        .into_inner();

    assert!(component.contains("export { run as run }"));
    assert!(source_map.unwrap().contains("\"version\":3,"));
    Ok(())
}
