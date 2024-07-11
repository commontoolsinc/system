/// The bytes of a Wasm Component that implements a JavaScript VM for
/// interpreting `common:module`-implementing JavaScript
pub static COMMON_JAVASCRIPT_INTERPRETER_WASM: &[u8] =
    include_bytes!(env!("COMMON_JAVASCRIPT_INTERPRETER_WASM_PATH"));
