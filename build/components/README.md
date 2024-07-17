This builder image is intended to build all of our Rust-based Wasm Component artifacts. Some or all of these artifacts are dependencies of other Rust crates, but since they must be built with a different target (`wasm32-wasip1`) and typically with the `cargo-component` tool, we use a builder image to compartmentalize the build.

In a typical workflow you might build the image like so:

```sh
docker build -f ./build/components/Dockerfile . -t component-builder
```

And then copy an artifact out of it like so:

```sh
docker run \
  -v /tmp/host:/host \
  component-builder \
  cp /build-root/target/wasm32-wasip1/release/common_thing.wasm /host
# Your artifact will be found in /tmp/host/common_thing.wasm
```