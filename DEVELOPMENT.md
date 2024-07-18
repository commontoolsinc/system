# Development

Notes on building and testing `system`.

## Dependencies

This project uses both Rust and Node.js. Both toolchains and dependencies will need to be installed.

### System

Install [protobuf-compiler] for your system.

### Rust

First, install [cargo] and [binstall] to set up the toolchain.
Then, install some Rust tools:

```bash
cargo binstall wit-deps-cli wasm-tools
```

### Node.js

First, install [node.js].
Then, install some node.js tools:

```bash
npm install -g @bytecodealliance/jco @bytecodealliance/componentize-js
```

## Tests

Once all dependencies are set up, run:

```bash
cargo test
```

## Optimizations

### Wasm Components

Currently, when you build some crates (like `common-runtime`), the `build.rs`
will invoke a Docker build step to produce some Wasm Component artifacts. This
can be quite slow in some environments.

If you can produce any needed Wasm artifacts as a pre-build step (either
compiling locally or perhaps pulling the artifacts from a cache), you can place
them in a `.wasm_cache` folder in the root of this workspace.

For an example of this, refer to the [`rust.yaml` Github Workflow](./.github/workflows/rust.yaml)

[cargo]: https://doc.rust-lang.org/cargo/getting-started/installation.html
[binstall]: https://github.com/cargo-bins/cargo-binstall
[node.js]: https://nodejs.org/en/learn/getting-started/how-to-install-nodejs
[protobuf-compiler]: https://grpc.io/docs/protoc-installation/
