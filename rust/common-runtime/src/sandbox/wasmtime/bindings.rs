#![allow(missing_docs)]

pub mod common_module {
    use wasmtime::component::bindgen;

    bindgen!({
      world: "common",
      path: "../../typescript/common/module/wit",
      async: true
    });
}

pub mod common_script {
    use wasmtime::component::bindgen;

    bindgen!({
      world: "common",
      path: "../../typescript/common/script/wit",
      async: true
    });
}
