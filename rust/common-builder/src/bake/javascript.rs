use super::{fs::write_file, Bake};
use crate::{BuilderError, JavaScriptBundler};
use async_trait::async_trait;
use bytes::Bytes;
use common_wit::{Target, WitTargetFileMap};
use tempfile::TempDir;
use tokio::process::Command;
use tokio::task::JoinSet;
use tracing::instrument;

/// A JavaScript-based [Bake] implementation,
/// using `jco`.
#[derive(Debug)]
pub struct JavaScriptBaker {}

#[async_trait]
impl Bake for JavaScriptBaker {
    #[instrument]
    async fn bake(&self, target: Target, source_code: Bytes) -> Result<Bytes, crate::BuilderError> {
        let workspace = TempDir::new()?;

        debug!(
            "Created temporary workspace in {}",
            workspace.path().display()
        );

        let bundled_source_code = tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current()
                .block_on(JavaScriptBundler::bundle_from_bytes(source_code))
        })
        .await??;

        let wasm_path = workspace.path().join("module.wasm");
        let js_path = workspace.path().join("module.js");

        debug!(?workspace, "Created temporary workspace");

        let mut writes = JoinSet::new();
        let wit_path = workspace.path().join("wit");

        writes.spawn(write_file(
            js_path.clone(),
            Bytes::from(bundled_source_code),
        ));

        writes.spawn({
            let wit_path = wit_path.clone();
            async move {
                WitTargetFileMap::from(target).write_to(&wit_path).await?;
                Ok(())
            }
        });

        while let Some(result) = writes.try_join_next() {
            result??;
            continue;
        }

        debug!(?workspace, "Populated temporary input files");

        let mut command = Command::new("jco");

        command
            .arg("componentize")
            .arg("-w")
            .arg(wit_path)
            .arg("-o")
            .arg(wasm_path.display().to_string())
            .arg(js_path.display().to_string());

        let child = command.spawn()?;
        let output = child.wait_with_output().await?;

        if !output.stderr.is_empty() {
            warn!("{}", String::from_utf8_lossy(&output.stderr));
        }
        if !output.status.success() {
            return Err(BuilderError::Internal("jco exited with a failure.".into()));
        }

        debug!("Finished building with jco");

        let wasm_bytes = tokio::fs::read(&wasm_path).await?;

        info!("Finished baking");

        Ok(wasm_bytes.into())
    }
}
