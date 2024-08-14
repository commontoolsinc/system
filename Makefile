.EXPORT_ALL_VARIABLES:

# Variables
export BASE_IMAGE_NAME = common-tools-base
export COMMON_MONOLITH_IMAGE_NAME = common-tools-monolith
export COMMON_BUILDER_IMAGE_NAME = common-builder
export COMMON_RUNTIME_IMAGE_NAME = common-runtime
export EXECUTION_IMAGE = "debian:bookworm-slim"

export TOOLCHAIN = stable
export NODE_VERSION = lts/*
export PLATFORM = ubuntu-latest
# export FEATURES =

export RUST_BACKTRACE = 1

# set WireIt variables
export WIREIT_LOGGER = metrics
# export WIREIT_CACHE = none

# Enable Docker BuildKit
export DOCKER_BUILDKIT = 1

# Enable Docker Compose BuildKit
export COMPOSE_DOCKER_CLI_BUILD = 1

########################################

# Default target
all: local

# Default target
local: build_rust list_outputs

# Target to build the docker images
docker: docker_base docker_monolith docker_builder docker_runtime

########################################

# Build the base image
docker_base: .docker_base.done
.docker_base.done: Dockerfile
	docker build -f Dockerfile \
	--tag $(BASE_IMAGE_NAME) . \
	--progress=plain \
	--cache-from=$(BASE_IMAGE_NAME) \
	--build-arg BASE_IMAGE=$(EXECUTION_IMAGE)

	touch .docker_base.done

# Build monolithic image
docker_monolith: .docker_monolith.done
.docker_monolith.done: rust/Dockerfile docker_base
	docker build -f rust/Dockerfile \
	--tag $(COMMON_MONOLITH_IMAGE_NAME) . \
	--progress=plain \
	--cache-from=$(BASE_IMAGE_NAME) \
	--build-arg BASE_IMAGE=$(BASE_IMAGE_NAME) \
	--build-arg EXECUTION_IMAGE=$(EXECUTION_IMAGE) \
	--build-arg FILES="target/release/*"

	touch .docker_monolith.done

# Build builder image
docker_builder: .docker_builder.done
.docker_builder.done: rust/Dockerfile docker_base
	docker build -f rust/Dockerfile \
	--tag $(COMMON_BUILDER_IMAGE_NAME) . \
	--progress=plain \
	--cache-from=$(BASE_IMAGE_NAME) \
	--build-arg BASE_IMAGE=$(BASE_IMAGE_NAME) \
	--build-arg EXECUTION_IMAGE=$(EXECUTION_IMAGE) \
	--build-arg FILES="target/release/builder" \
	--build-arg BINARY_PATH="target/release/builder" \
	--build-arg EXPOSED_PORT=8080

	touch .docker_builder.done

# Build runtime image
docker_runtime: .docker_runtime.done
.docker_runtime.done: rust/Dockerfile docker_base
	docker build -f rust/Dockerfile \
	--tag $(COMMON_RUNTIME_IMAGE_NAME) . \
	--progress=plain \
	--cache-from=$(BASE_IMAGE_NAME) \
	--build-arg BASE_IMAGE=$(BASE_IMAGE_NAME) \
	--build-arg EXECUTION_IMAGE=$(EXECUTION_IMAGE) \
	--build-arg FILES="target/release/runtime" \
	--build-arg BINARY_PATH="target/release/runtime" \
	--build-arg EXPOSED_PORT=8081

	touch .docker_runtime.done

########################################

# Target to install npm dependencies and build
build_typescript: .build_typescript.done .build_wit.done
.build_typescript.done: $(wildcard typescript/**/*.ts) $(wildcard typescript/**/package-lock.json) $(wildcard typescript/**/package.json) build_wit
	cd typescript && npm ci && npm run build --workspaces
	touch .build_typescript.done

# Target to perform rust build
build_rust: .build_rust.done
.build_rust.done: $(wildcard **/Cargo.toml) $(wildcard **/Cargo.lock) $(wildcard **/deps.lock) $(wildcard rust/**/*.rs) $(wildcard rust/common-test-fixtures/fixtures/**/*.js) build_wit
	cargo build --release
	touch .build_rust.done

build_wit: .build_wit.done
.build_wit.done:
	./wit/wit-tools.sh deps
	touch .build_wit.done

# Target to list rust build outputs
list_outputs: .build_rust.done
	ls -alvh target/release

########################################

# Phony targets
.PHONY: all local docker docker_base docker_monolith docker_builder docker_runtime build_typescript build_rust list_outputs build_wit
