use crate::{builder::Builder, error::Error, storage::PersistedHashStorage};
use ct_protos::{builder::builder_server::BuilderServer, MAX_MESSAGE_SIZE};
use tokio::net::TcpListener;
use tonic::transport::Server as TonicServer;

/// Start the Common Builder server, serving gRPC on `grpc_listener`.
pub async fn serve(grpc_listener: TcpListener) -> Result<(), Error> {
    let storage = PersistedHashStorage::temporary()?;
    let builder = Builder::new(storage);
    serve_grpc(builder, grpc_listener).await?;
    Ok(())
}

/// Start the Common Builder server, listening to incoming connections on the
/// provided [`TcpListener`]
async fn serve_grpc(builder: Builder, listener: TcpListener) -> Result<(), Error> {
    let incoming_stream = async_stream::stream! {
        loop {
            let (stream, _) = listener.accept().await?;
            yield Ok::<_, std::io::Error>(stream);
        }
    };
    let builder_server = BuilderServer::new(builder)
        .max_encoding_message_size(MAX_MESSAGE_SIZE)
        .max_decoding_message_size(MAX_MESSAGE_SIZE);

    TonicServer::builder()
        .add_service(builder_server)
        .serve_with_incoming(incoming_stream)
        .await
        .map_err(|error| Error::Internal(format!("Failed to start server: {error}")))?;

    Ok(())
}
