# common-wasi

WASI 0.2 implementation for Common components.

A fork of [wasmtime_wasi](https://docs.rs/wasmtime-wasi) with the following changes:

* All functionality for the interfaces below are stubbed; functions are provided, but executing them results in an error:
  * `wasi:http`
  * `wasi:filesystem`
  * `wasi:sockets`
* `wasi:http` (stubbed) implementation included, instead of a separate crate ([wasmtime_wasi_http](https://docs.rs/wasmtime-wasi-http)).
* `WasiCtxBuilder` cannot inherit host streams, environment, or arguments.
