# Variables
BASE_IMAGE_NAME = common-tools/base:latest
COMMON_BUILDER_IMAGE_NAME = common-builder:latest
COMMON_RUNTIME_IMAGE_NAME = common-runtime:latest
TOOLCHAIN = stable
NODE_VERSION = lts/*
PLATFORM = ubuntu-latest
FEATURES =

export RUST_BACKTRACE=1
# set WireIt variables
export WIREIT_LOGGER=metrics
export WIREIT_CACHE=none

# Enable Docker BuildKit
export DOCKER_BUILDKIT=1

# .PHONY is a special target that tells make that the listed targets are not files
.PHONY: all base common-builder common-runtime lints build-wasm32-unknown-unknown build-wasm-components native-target-tests setup-rust setup-node npm-deps rust-deps os-packages clean

docker: base

local: build-wasm-components

# Build the base image
base:
	docker build -f Dockerfile -t $(BASE_IMAGE_NAME) . --progress=plain --cache-from=$(BASE_IMAGE_NAME)
	# get the digest and assign to Variable
	BASE_IMAGE_DIGEST=$(shell docker inspect --format='{{index .RepoDigests 0}}' $(BASE_IMAGE_NAME))
	# tag with digest
	docker tag $(BASE_IMAGE_NAME) $(BASE_IMAGE_DIGEST)

# Build the common-builder image
common-builder:
	@$(MAKE) base
	docker build --build-arg BASE_IMAGE=$(BASE_IMAGE_NAME) -f rust/common-builder/Dockerfile -t $(COMMON_BUILDER_IMAGE_NAME) --cache-from=$(COMMON_BUILDER_IMAGE_NAME) .

# Build the common-runtime image
common-runtime:
	@$(MAKE) base
	docker build --build-arg BASE_IMAGE=$(BASE_IMAGE_NAME) -f rust/common-runtime/Dockerfile -t $(COMMON_RUNTIME_IMAGE_NAME) --cache-from=$(COMMON_RUNTIME_IMAGE_NAME) .

# Clean up the images
clean:
	docker rmi $(COMMON_BUILDER_IMAGE_NAME) $(COMMON_RUNTIME_IMAGE_NAME) $(BASE_IMAGE_NAME)

setup: setup-os-packages rust-deps npm-deps

build-common: setup build-common-javascript-interpreter
	@echo "Building common..."
	cargo build --release

build-common-javascript-interpreter: setup
	@echo "Building common-javascript-interpreter..."
	export COMMON_JAVASCRIPT_INTERPRETER_WASM_PATH=$(shell pwd)/target/wasm32-unknown-unknown/release/common_javascript_interpreter.wasm
	cargo build -p common-javascript-interpreter --release
	cd rust/common-javascript-interpreter && \
	cargo component check

build-wasm32-unknown-unknown: setup build-common build-common-javascript-interpreter
	@echo "Building wasm32-unknown-unknown..."
	rustup target add wasm32-unknown-unknown
	cargo build --release

build-wasm-components: setup build-common build-wasm32-unknown-unknown
	@echo "Building Wasm Components..."common_javascript_interpreter.wasm
	cargo component build -p common-javascript-interpreter --release

rust-deps:
	@echo "Installing rust-deps..."
	rustup toolchain install ${TOOLCHAIN}
	rustup +${TOOLCHAIN} component add clippy
	rustup +${TOOLCHAIN} component add rustfmt
	cargo install cargo-binstall
	cargo +${TOOLCHAIN} binstall cargo-component
	cargo +${TOOLCHAIN} binstall cargo-nextest
	cargo +${TOOLCHAIN} binstall wit-deps-cli
	cargo +${TOOLCHAIN} binstall wasm-tools

OS:=$(shell uname)
setup-os-packages:
	@echo "Installing OS packages..."
	@echo "OS: $(OS)"

	@if [ "$(OS)" = "Darwin" ]; then \
		brew install protobuf; \
	elif [ "$(OS)" = "Linux" ]; then \
		apt install -y protobuf-compiler; \
		apt install -y libprotobuf-dev; \
		apt install -y pkg-config; \
		apt install -y libssl-dev; \
	else \
		echo "Unsupported OS: $(OS)"; \
		exit 1; \
	fi
	# ifeq ("($(OS),Darwin)")
	# 	brew install protobuf
	# else ifeq ($(OS),Linux)
	# 	apt install -y protobuf-compiler
	# 	apt install -y libprotobuf-dev
	# 	apt install -y pkg-config
	# 	apt install -y libssl-dev
	# else
	# 	echo "Unsupported OS: $(OS)"
	# 	exit 1

npm-deps:
	@echo "Installing NPM dependencies..."
	npm install -g @bytecodealliance/jco
	npm install -g @bytecodealliance/componentize-js
	npm install -g wireit
	cd typescript && \
	npm ci && \
	npm run build
