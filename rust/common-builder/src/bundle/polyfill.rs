use super::wasi_shim::WASI_MAPPINGS;
use anyhow::Result;
use js_component_bindgen::{transpile, InstantiationMode, TranspileOpts, Transpiled};
use std::collections::HashMap;
use wasmtime_environ::component::Export as WasmtimeExport;

use crate::{BuilderError, JavaScriptBundler};

/// Type of instantiation for a WASM module.
pub enum Instantiation {
    /// The WASM module is automatically instantiated.
    Automatic,
    /// The WASM module needs to be manually instantiated.
    Manual,
}

/// Type of WASM exports used in [Artifacts].
#[derive(Debug)]
pub enum ExportType {
    /// Export is a function.
    Function,
    /// Export is an instance.
    Instance,
}

/// Representing all generated artifacts from a [polyfill]
/// invocation containing files, imports and exports.
#[derive(Debug)]
pub struct Artifacts {
    /// Sequence of file tuples, containing the filename and bytes.
    pub files: Vec<(String, Vec<u8>)>,
    /// Sequence of import function names.
    pub imports: Vec<String>,
    /// Sequence of export function names and their types.
    pub exports: Vec<(String, ExportType)>,
}

/// Helper struct to map wasi shim imports.
struct Mappings(HashMap<String, String>);

impl<S> FromIterator<(S, S)> for Mappings
where
    S: ToString,
{
    fn from_iter<T: IntoIterator<Item = (S, S)>>(iter: T) -> Self {
        let mut map = HashMap::default();
        for (k, v) in iter.into_iter() {
            map.insert(k.to_string(), v.to_string());
        }
        Mappings(map)
    }
}

impl From<Mappings> for HashMap<String, String> {
    fn from(value: Mappings) -> Self {
        value.0
    }
}

/// Take a WASM component and generate
/// a suite of [Artifacts] resulting in a polyfilled version.
pub async fn polyfill(
    name: &str,
    component: Vec<u8>,
    instantiation: Option<Instantiation>,
) -> Result<Artifacts, BuilderError> {
    let wasi_mappings = Mappings::from_iter(WASI_MAPPINGS.into_iter()).into();

    let options = TranspileOpts {
        name: name.to_owned(),
        map: Some(wasi_mappings),
        no_typescript: true,
        instantiation: instantiation
            .unwrap_or_else(|| Instantiation::Automatic)
            .into(),
        import_bindings: None,
        no_nodejs_compat: true,
        base64_cutoff: 1024,
        tla_compat: false,
        valid_lifting_optimization: false,
        tracing: false,
        no_namespaced_exports: true,
        multi_memory: false,
    };

    let mut artifacts: Artifacts = transpile(&component, options)
        .map(|transpiled| transpiled.into())
        .map_err(|error| anyhow::anyhow!("{}", error))?;

    // Rewrite JS files, populating the wasi shims.
    artifacts.files = {
        let mut shimmed_files = vec![];
        for (file_name, bytes) in artifacts.files.into_iter() {
            let new_bytes = if file_name.ends_with(".js") {
                JavaScriptBundler::bundle_from_bytes_sync(bytes::Bytes::from(bytes))
                    .await?
                    .into_bytes()
            } else {
                bytes
            };
            shimmed_files.push((file_name, new_bytes));
        }
        shimmed_files
    };

    // Filter out the "common:wasi-shim/" imports that were
    // just bundled into the JavaScript.
    artifacts.imports = artifacts
        .imports
        .into_iter()
        .filter(|specifier| !specifier.starts_with("common:wasi-shim/"))
        .collect();

    Ok(artifacts)
}

impl From<Instantiation> for Option<InstantiationMode> {
    fn from(value: Instantiation) -> Self {
        match value {
            Instantiation::Automatic => None,
            Instantiation::Manual => Some(InstantiationMode::Async),
        }
    }
}

impl From<Transpiled> for Artifacts {
    fn from(value: Transpiled) -> Self {
        Artifacts {
            imports: value.imports,
            exports: value
                .exports
                .into_iter()
                .map(|(name, export)| {
                    let export_type = match export {
                        WasmtimeExport::LiftedFunction { .. } => ExportType::Function,
                        WasmtimeExport::Instance { .. } => ExportType::Instance,
                        _ => panic!("Unexpected export type"),
                    };
                    (name, export_type)
                })
                .collect(),
            files: value.files,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Bake, JavaScriptBaker};
    use anyhow::Result;
    use common_test_fixtures::sources::common::BASIC_MODULE_JS;
    use common_tracing::common_tracing;
    use common_wit::Target;

    #[tokio::test]
    #[common_tracing]
    async fn it_polyfills_wasm_component() -> Result<()> {
        let baker = JavaScriptBaker {};
        let wasm_component = baker
            .bake(Target::CommonModule, BASIC_MODULE_JS.into())
            .await?;

        let artifacts = polyfill("mymodule", wasm_component.to_vec(), None).await?;

        // Check imports
        assert!(artifacts.imports.contains(&"common:data/types".to_string()));
        assert!(artifacts.imports.contains(&"common:io/state".to_string()));
        assert!(!artifacts.imports.contains(&"common:wasi-shim/".to_string()));

        // Check files
        assert_eq!(
            artifacts
                .files
                .iter()
                .map(|(name, _)| name.to_owned())
                .collect::<Vec<_>>(),
            vec![
                "mymodule.core.wasm".to_string(),
                "mymodule.core2.wasm".to_string(),
                "mymodule.core3.wasm".to_string(),
                "mymodule.js".to_string()
            ]
        );

        let js_out = String::from_utf8(artifacts.files[3].1.clone()).unwrap();
        assert!(!js_out.contains("common:wasi-stub"));
        assert!(js_out.contains("const monotonicClock ="));

        Ok(())
    }
}
