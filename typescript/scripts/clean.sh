#!/usr/bin/env bash

# Cleans the `typescript` directory of build artifacts.

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

TYPESCRIPT_DIR="$SCRIPT_DIR/.."

rm -rf "$TYPESCRIPT_DIR/node_modules"
rm -rf "$TYPESCRIPT_DIR/common/data/lib"
rm -rf "$TYPESCRIPT_DIR/common/data/tsconfig.tsbuildinfo"
rm -rf "$TYPESCRIPT_DIR/common/function/lib"
rm -rf "$TYPESCRIPT_DIR/common/function/tsconfig.tsbuildinfo"
rm -rf "$TYPESCRIPT_DIR/common/io/lib"
rm -rf "$TYPESCRIPT_DIR/common/io/tsconfig.tsbuildinfo"
