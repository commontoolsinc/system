#![allow(missing_docs)]

pub mod common_function {
    use wasmtime::component::bindgen;

    bindgen!({
      world: "module",
      path: "../../typescript/common/function/wit",
      async: true
    });
}

pub mod common_function_vm {
    use wasmtime::component::bindgen;

    bindgen!({
      world: "virtual-module",
      path: "../../typescript/common/function/wit",
      async: true
    });
}
