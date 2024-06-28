#![warn(missing_docs)]
//! Utilities for compiling/bundling JavaScript into
//! a single source.

#[macro_use]
extern crate tracing;

use anyhow::{anyhow, Result};
use bytes::Bytes;
use deno_emit::{
    bundle, BundleOptions, BundleType, EmitOptions, LoadFuture, LoadOptions, Loader,
    ModuleSpecifier, SourceMapOption, TranspileOptions,
};
use deno_graph::source::LoadResponse;
use reqwest::Client;
use url::Url;

const ROOT_MODULE_URL: &str = "bundler:root";
const ROOT_MODULE_SCHEME: &str = "bundler";

struct JavaScriptLoader {
    root: Option<Bytes>,
    client: Client,
}

impl JavaScriptLoader {
    pub fn new(root: Option<Bytes>) -> Self {
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
                            anyhow!("Attempted to load root module, but no root was specified!")
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
                source_map: SourceMapOption::None,
                source_map_file: None,
                inline_sources: false,
                remove_comments: true,
            },
            emit_ignore_directives: false,
            minify: false,
        }
    }

    /// Bundle a JavaScript module via URL.
    pub async fn bundle_from_url(url: Url) -> Result<String> {
        let mut loader = JavaScriptLoader::new(None);
        let emit = bundle(url, &mut loader, None, Self::bundle_options()).await?;
        Ok(emit.code)
    }

    /// Bundle a JavaScript module from bytes.
    pub async fn bundle_from_bytes(module: Bytes) -> Result<String> {
        let mut loader = JavaScriptLoader::new(Some(module));
        let emit = bundle(
            Url::parse(ROOT_MODULE_URL)?,
            &mut loader,
            None,
            Self::bundle_options(),
        )
        .await?;
        Ok(emit.code)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::JavaScriptBundler;
    use anyhow::Result;
    use common_test_fixtures::EsmTestServer;
    use url::Url;

    fn assert_math_bundle(bundle: &str) {
        assert!(!bundle.is_empty());
        assert!(bundle.contains("function add"));
        assert!(bundle.contains("function mult"));
    }

    #[tokio::test]
    async fn it_bundles_javascript_from_url() -> Result<()> {
        common_tracing::initialize_tracing();
        let mut server = EsmTestServer::default();
        let addr = server.start().await?;
        let candidate = Url::parse(&format!("http://{}/math/index.js", addr))?;
        let bundle = JavaScriptBundler::bundle_from_url(candidate).await?;

        assert_math_bundle(&bundle);

        Ok(())
    }

    #[tokio::test]
    async fn it_bundles_typescript_from_url() -> Result<()> {
        common_tracing::initialize_tracing();
        let mut server = EsmTestServer::default();
        let addr = server.start().await?;
        let candidate = Url::parse(&format!("http://{}/math/index.ts", addr))?;
        let bundle = JavaScriptBundler::bundle_from_url(candidate).await?;

        assert_math_bundle(&bundle);

        Ok(())
    }

    #[tokio::test]
    async fn it_bundles_javascript_from_bytes() -> Result<()> {
        common_tracing::initialize_tracing();
        let mut server = EsmTestServer::default();
        let addr = server.start().await?;
        let candidate = format!(
            r#"export * from "http://{}/math/index.js";
"#,
            addr
        );
        let bundle = JavaScriptBundler::bundle_from_bytes(candidate.into()).await?;

        assert_math_bundle(&bundle);

        Ok(())
    }

    #[tokio::test]
    async fn it_skips_common_modules_when_bundling() -> Result<()> {
        common_tracing::initialize_tracing();
        let candidate = r#"
import { read, write } from "common:io/state@0.0.1";

// Note: must use imports else they are tree-shaken
// Caveat: cannot re-export built-ins as it provokes bundling
console.log(read, write);
"#
        .to_string();

        let bundle = JavaScriptBundler::bundle_from_bytes(candidate.into())
            .await
            .map_err(|error| {
                error!("{}", error);
                error
            })
            .unwrap();

        debug!("{bundle}");

        assert!(bundle.contains("import { read, write } from \"common:io/state@0.0.1\""));

        Ok(())
    }
}
