use anyhow::Result;
use js_component_bindgen::{transpile, InstantiationMode, TranspileOpts, Transpiled};
use std::collections::HashMap;
use wasmtime_environ::component::Export as WasmtimeExport;

//pub type Mappings = HashMap<String, String>;
pub struct Mappings {
    map: HashMap<String, String>,
}

impl Mappings {
    pub fn into_map(self) -> HashMap<String, String> {
        self.map
    }
}

impl From<HashMap<String, String>> for Mappings {
    fn from(map: HashMap<String, String>) -> Self {
        Mappings { map }
    }
}

impl From<Mappings> for HashMap<String, String> {
    fn from(value: Mappings) -> Self {
        value.into_map()
    }
}

impl<S> FromIterator<(S, S)> for Mappings
where
    S: ToString,
{
    fn from_iter<T: IntoIterator<Item = (S, S)>>(iter: T) -> Self {
        let mut map = HashMap::default();
        for (k, v) in iter.into_iter() {
            map.insert(k.to_string(), v.to_string());
        }
        Mappings { map }
    }
}
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

/// Take a WASM component with optional mappings and generate
/// a suite of [Artifacts] resulting in a polyfilled version.
pub fn polyfill(
    name: &str,
    component: Vec<u8>,
    mappings: Option<Mappings>,
    instantiation: Option<Instantiation>,
) -> Result<Artifacts> {
    let options = TranspileOpts {
        name: name.to_owned(),
        map: mappings.map(|m| m.into()),
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

    transpile(&component, options)
        .map(|transpiled| transpiled.into())
        .map_err(|error| anyhow::anyhow!("{}", error))
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
    use crate::{Bake, JavaScriptBaker};
    use super::*;
    use anyhow::Result;
    use bytes::Bytes;
    use common_wit::WitTarget;

    const WASI_MAPPINGS: [(&str, &str); 7] = [
        ("wasi:cli/*", "/wasi-shim/cli.js#*"),
        ("wasi:clocks/*", "/wasi-shim/clocks.js#*"),
        ("wasi:filesystem/*", "/wasi-shim/filesystem.js#*"),
        ("wasi:http/*", "/wasi-shim/http.js#*"),
        ("wasi:io/*", "/wasi-shim/io.js#*"),
        ("wasi:random/*", "/wasi-shim/random.js#*"),
        ("wasi:sockets/*", "/wasi-shim/sockets.js#*"),
    ];

    #[tokio::test]
    async fn it_polyfills_wasm_component() -> Result<()> {
        let source_code = Bytes::from(
            r#"
import { read, write } from 'common:io/state@0.0.1';
export class Body {
    run() {
        console.log('Running!');
    }
}
export const module = {
  Body,

  create() {
      console.log('Creating!');
      return new Body();
  }
};"#,
        );
        let baker = JavaScriptBaker {};
        let wasm_component = baker.bake(WitTarget::CommonModule, source_code).await?;

        let wasi_mappings = Mappings::from_iter(
            WASI_MAPPINGS
                .into_iter()
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect::<Vec<(String, String)>>(),
        );

        let artifacts = polyfill(
            "mymodule",
            wasm_component.to_vec(),
            Some(wasi_mappings),
            None,
        )?;

        assert!(artifacts.imports.contains(&"common:data/types".to_string()));
        assert!(artifacts.imports.contains(&"common:io/state".to_string()));
        assert_eq!(artifacts.files.into_iter().map(|(name, _)| name).collect::<Vec<_>>(), vec![
            "mymodule.core.wasm".to_string(),
            "mymodule.core2.wasm".to_string(),
            "mymodule.core3.wasm".to_string(),
            "mymodule.js".to_string()
        ]);

        Ok(())
    }
}
