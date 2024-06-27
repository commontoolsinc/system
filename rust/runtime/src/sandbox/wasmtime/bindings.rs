#![allow(missing_docs)]

use wasmtime::component::bindgen;

bindgen!({
  world: "common",
  path: "../../typescript/common/module/wit"
});
