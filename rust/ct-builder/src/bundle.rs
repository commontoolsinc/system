//! Utilities for compiling/bundling JavaScript into
//! a single source.

use anyhow::anyhow;
use deno_emit::{
    bundle, BundleOptions, BundleType, EmitOptions, LoadFuture, LoadOptions, Loader,
    ModuleSpecifier, SourceMapOption, TranspileOptions,
};
use deno_graph::source::LoadResponse;
use reqwest::Client;
use url::Url;

use crate::{artifact::Artifact, Error};

// Root module must have `.tsx` in order to be
// interprete as Typescript/JSX.
const ROOT_MODULE_URL: &str = "bundler:root.tsx";
const ROOT_MODULE_SCHEME: &str = "bundler";

struct JavaScriptLoader {
    root: Option<Vec<u8>>,
    client: Client,
}

impl JavaScriptLoader {
    pub fn new(root: Option<Vec<u8>>) -> Self {
        Self {
            root,
            client: Client::new(),
        }
    }
}

impl Loader for JavaScriptLoader {
    fn load(&self, specifier: &ModuleSpecifier, _options: LoadOptions) -> LoadFuture {
        let root = self.root.clone();
        let client = self.client.clone();
        let specifier = specifier.clone();

        debug!("Attempting to load '{}'", specifier);

        Box::pin(async move {
            match specifier.scheme() {
                ROOT_MODULE_SCHEME => Ok(Some(LoadResponse::Module {
                    content: root
                        .ok_or_else(|| {
                            Error::InvalidConfiguration(
                                "Attempted to load root module, but no root was specified!".into(),
                            )
                        })?
                        .to_vec()
                        .into(),
                    specifier,
                    maybe_headers: None,
                })),
                "common" => Ok(Some(LoadResponse::External {
                    specifier: specifier.clone(),
                })),
                "http" | "https" => {
                    let response = client.get(specifier.as_str()).send().await?;
                    let headers = response.headers().to_owned();
                    let bytes = response.bytes().await?;
                    let content = bytes.to_vec().into();

                    trace!("Loaded remote module: {}", String::from_utf8_lossy(&bytes));
                    Ok(Some(LoadResponse::Module {
                        content,
                        specifier,
                        maybe_headers: Some(
                            headers
                                .into_iter()
                                .filter_map(|(h, v)| {
                                    h.map(|header| {
                                        (
                                            header.to_string(),
                                            v.to_str().unwrap_or_default().to_string(),
                                        )
                                    })
                                })
                                .collect(),
                        ),
                    }))
                }
                "node" | "npm" => Err(anyhow!(
                    "Could not import '{specifier}'. Node.js and NPM modules are not supported."
                )),
                _ => Err(anyhow!(
                    "Could not import '{specifier}'. Unrecognize specifier format.'"
                )),
            }
        })
    }
}

/// A namespace for functions that resolves a JavaScript source
/// file's dependencies and bundles into a single artifact.
pub struct JavaScriptBundler {}

impl JavaScriptBundler {
    fn bundle_options() -> BundleOptions {
        BundleOptions {
            bundle_type: BundleType::Module,
            transpile_options: TranspileOptions::default(),
            emit_options: EmitOptions {
                source_map: SourceMapOption::Separate,
                source_map_file: None,
                source_map_base: None,
                inline_sources: false,
                remove_comments: true,
            },
            emit_ignore_directives: false,
            minify: false,
        }
    }

    /// Bundle a JavaScript module via URL.
    pub async fn bundle_from_url(url: Url) -> Result<Artifact, Error> {
        let mut loader = JavaScriptLoader::new(None);
        let emit = bundle(url, &mut loader, None, Self::bundle_options()).await?;
        Ok(emit.into())
    }

    /// Bundle a JavaScript module from bytes.
    pub async fn bundle_from_bytes(module: Vec<u8>) -> Result<Artifact, Error> {
        let mut loader = JavaScriptLoader::new(Some(module));
        let emit = bundle(
            Url::parse(ROOT_MODULE_URL).map_err(|error| Error::Internal(format!("{error}")))?,
            &mut loader,
            None,
            Self::bundle_options(),
        )
        .await?;
        Ok(emit.into())
    }

    /// Spawns a blocking bundle operation on a thread dedicated to blocking
    /// operations. This is needed in cases where bundling is taking place e.g.,
    /// within a web server.
    pub async fn bundle_from_bytes_sync(source_code: Vec<u8>) -> Result<Artifact, Error> {
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current()
                .block_on(JavaScriptBundler::bundle_from_bytes(source_code))
        })
        .await?
    }
}

#[cfg(test)]
pub mod tests {
    use crate::JavaScriptBundler;
    use anyhow::Result;
    use ct_test_fixtures::EsmTestServer;
    use ct_tracing::ct_tracing;
    use url::Url;

    fn assert_math_bundle(bundle: &str) {
        assert!(!bundle.is_empty());
        assert!(bundle.contains("function add"));
        assert!(bundle.contains("function mult"));
    }

    #[tokio::test]
    #[ct_tracing]
    async fn it_bundles_javascript_from_url() -> Result<()> {
        let mut server = EsmTestServer::default();
        let addr = server.start().await?;
        let candidate = Url::parse(&format!("http://{}/math/index.js", addr))?;
        let bundle = JavaScriptBundler::bundle_from_url(candidate).await?;

        assert_math_bundle(&bundle.component);

        Ok(())
    }

    #[tokio::test]
    #[ct_tracing]
    async fn it_bundles_typescript_from_url() -> Result<()> {
        let mut server = EsmTestServer::default();
        let addr = server.start().await?;
        let candidate = Url::parse(&format!("http://{}/math/index.ts", addr))?;
        let bundle = JavaScriptBundler::bundle_from_url(candidate).await?;

        assert_math_bundle(&bundle.component);

        Ok(())
    }

    #[tokio::test]
    #[ct_tracing]
    async fn it_bundles_javascript_from_bytes() -> Result<()> {
        let mut server = EsmTestServer::default();
        let addr = server.start().await?;
        let candidate = format!(
            r#"export * from "http://{}/math/index.js";
"#,
            addr
        );
        let bundle = JavaScriptBundler::bundle_from_bytes(candidate.into()).await?;

        assert_math_bundle(&bundle.component);

        Ok(())
    }

    #[tokio::test]
    #[ct_tracing]
    async fn it_bundles_typescript_from_bytes() -> Result<()> {
        let candidate = r#"
export const add = function add(x: number, y: number): number {
  return x + y;
}
"#
        .to_string();

        let bundle = JavaScriptBundler::bundle_from_bytes(candidate.into()).await?;
        assert!(bundle.component.contains("function add"));

        Ok(())
    }

    #[tokio::test]
    #[ct_tracing]
    async fn it_skips_common_esm_modules_when_bundling() -> Result<()> {
        let candidate = r#"
import { read, write } from "common:io/state@0.0.1";

// Note: must use imports else they are tree-shaken
// Caveat: cannot re-export built-ins as it provokes bundling
console.log(read, write);
"#
        .to_string();

        let bundle = JavaScriptBundler::bundle_from_bytes(candidate.into()).await?;

        assert!(bundle
            .component
            .contains("import { read, write } from \"common:io/state@0.0.1\""));

        Ok(())
    }
}
