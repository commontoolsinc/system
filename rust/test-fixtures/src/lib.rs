#![warn(missing_docs)]
//! A static server for serving various types of ES modules
//! for testing.

use anyhow::Result;
use axum::Router;
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
};
use tokio::{net::TcpListener, task::JoinHandle};
use tower_http::services::ServeDir;

/// Manages a static server for use in builders and bundlers,
/// serving various types of ES modules.
pub struct EsmTestServer {
    handle: Option<JoinHandle<()>>,
    dir: PathBuf,
}

impl EsmTestServer {
    /// Create a new [ESMTestServer], serving the
    /// `dir` directory at the host HTTP root "/".
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            handle: None,
            dir: dir.as_ref().into(),
        }
    }

    /// Start the static server on `port`. Upon success, a
    /// [SocketAddr] is returned of the static server.
    pub async fn start_with_port(&mut self, port: u16) -> Result<SocketAddr> {
        let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), port)).await?;
        let addr = listener.local_addr()?;
        let serve_dir = ServeDir::new(&self.dir);
        self.handle = Some(tokio::spawn(async {
            let app = Router::new().nest_service("/", serve_dir);
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap();
        }));
        Ok(addr)
    }

    /// Start the static server, using an available port implicitly.
    /// See [ESMTestServer::start_with_port] for more details.
    pub async fn start(&mut self) -> Result<SocketAddr> {
        self.start_with_port(0).await
    }

    /// Terminates the static server. Called automatically when
    /// [ESMTestServer] is dropped.
    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

impl Default for EsmTestServer {
    fn default() -> Self {
        // The crate root is used as PWD when running
        // cargo tests, so in most cases this should
        // serve as a good default. TBD what ways
        // tests can run where this fails.
        Self::new("../test-fixtures/fixtures")
    }
}

impl Drop for EsmTestServer {
    fn drop(&mut self) {
        self.stop()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::fs;

    #[tokio::test]
    async fn it_serves_content_from_static_dir() -> Result<()> {
        let mut server = EsmTestServer::default();
        let addr = server.start().await?;
        let url = format!("http://{}/math/index.js", addr);
        let module = reqwest::get(url).await?.text().await?;

        assert_eq!(module, fs::read_to_string("fixtures/math/index.js").await?);
        Ok(())
    }
}
