.EXPORT_ALL_VARIABLES:

# Variables
export BASE_IMAGE_NAME = common-tools-base
export COMMON_MONOLITH_IMAGE_NAME = common-tools-monolith
export COMMON_BUILDER_IMAGE_NAME = common-builder
export COMMON_RUNTIME_IMAGE_NAME = common-runtime
export RUNTIME_BASE_IMAGE = "debian:latest"

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
docker_base: .docker_base_done
.docker_base_done: Dockerfile.base
	docker build -f Dockerfile.base \
	--tag $(BASE_IMAGE_NAME) . \
	--progress=plain \
	--cache-from=$(BASE_IMAGE_NAME) \
	--build-arg BASE_IMAGE=$(RUNTIME_BASE_IMAGE)

	touch .docker_base_done

# Build monolithic image
docker_monolith: .docker_monolith_done
.docker_monolith_done: Dockerfile docker_base
	docker build -f Dockerfile \
	--tag $(COMMON_MONOLITH_IMAGE_NAME) . \
	--progress=plain \
	--cache-from=$(BASE_IMAGE_NAME) \
	--build-arg BASE_IMAGE=$(BASE_IMAGE_NAME) \
	--build-arg RUNTIME_BASE_IMAGE=$(RUNTIME_BASE_IMAGE) \
	--build-arg FILES="target/release/*"

	touch .docker_monolith_done

# Build builder image
docker_builder: .docker_builder_done
.docker_builder_done: Dockerfile docker_base
	docker build -f Dockerfile \
	--tag $(COMMON_BUILDER_IMAGE_NAME) . \
	--progress=plain \
	--cache-from=$(BASE_IMAGE_NAME) \
	--build-arg BASE_IMAGE=$(BASE_IMAGE_NAME) \
	--build-arg RUNTIME_BASE_IMAGE=$(RUNTIME_BASE_IMAGE) \
	--build-arg FILES="target/release/builder" \
	--build-arg BINARY_PATH="target/release/builder" \
	--build-arg EXPOSED_PORT=8080

	touch .docker_builder_done

# Build runtime image
docker_runtime: .docker_runtime_done
.docker_runtime_done: Dockerfile docker_base
	docker build -f Dockerfile \
	--tag $(COMMON_RUNTIME_IMAGE_NAME) . \
	--progress=plain \
	--cache-from=$(BASE_IMAGE_NAME) \
	--build-arg BASE_IMAGE=$(BASE_IMAGE_NAME) \
	--build-arg RUNTIME_BASE_IMAGE=$(RUNTIME_BASE_IMAGE) \
	--build-arg FILES="target/release/runtime" \
	--build-arg BINARY_PATH="target/release/runtime" \
	--build-arg EXPOSED_PORT=8081

	touch .docker_runtime_done

########################################

# Target to install npm dependencies and build
build_typescript: .build_typescript_done
.build_typescript_done: $(wildcard typescript/**/*.ts) $(wildcard typescript/**/package-lock.json) $(wildcard typescript/**/package.json)
	cd typescript && npm ci && npm run build
	touch .build_typescript_done

# Target to perform rust build
build_rust: .build_rust_done
.build_rust_done: $(wildcard rust/**/Cargo.toml) $(wildcard rust/**/*.rs) $(wildcard rust/common-test-fixtures/fixtures/**/*.js)
	cargo build --release
	touch .build_rust_done

# Target to list rust build outputs
list_outputs: .build_rust_done
	ls -alvh target/release

########################################

# Phony targets
.PHONY: all local docker docker_base docker_monolith docker_builder docker_runtime build_typescript build_rust list_outputs
