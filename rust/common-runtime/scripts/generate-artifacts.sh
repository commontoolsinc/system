#!/usr/bin/env bash

set -exuo pipefail

pushd ../../

ARTIFACT=common_javascript_interpreter.wasm
BUILD_ROOT=/build-root/target/wasm32-wasip1/release

docker build -f ./build/components/Dockerfile . -t component-builder
docker run \
  -v $OUT_DIR:/host \
  component-builder \
  bash -c "cp $BUILD_ROOT/$ARTIFACT /host && chown -R $(id -u $USER):$(id -g $USER) /host/$ARTIFACT && chmod +r /host/$ARTIFACT"

if [ ! -f $OUT_DIR/common_javascript_interpreter.wasm ]; then
  echo "Expected artifact dependency 'common_javascript_interpreter.wasm' was not found"
  exit 1
fi

popd