# common-runtime 

The Common Runtime is responsible for instantiating a Common Recipe and managing its lifecycle.

## Building

### Environment variables

* `CARGO_COMPONENT_CACHE_DIR`: This value is propagated to underlying `cargo component build` commands for wasm artifacts, setting the cache directory used by cargo component. By default, this will be set to `[cache_dir]/cargo-component/cache` which should be sufficient, but for nix builds we need to build inside the tree.

[cache-dir]: https://docs.rs/dirs/latest/dirs/fn.cache_dir.html
