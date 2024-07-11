#!/usr/bin/env bash

set -euo pipefail

pushd ../../

docker build -f ./build/components/Dockerfile . -t component-builder
docker run \
  -v $OUT_DIR:/host \
  component-builder \
  bash -c "cp /build-root/target/wasm32-wasip1/release/common_javascript_interpreter.wasm /host && chown -R $(id -u $USER):$(id -g $USER) /host"

popd