# Variables
BASE_IMAGE_NAME = common-tools-base:latest
COMMON_BUILDER_IMAGE_NAME = common-builder:latest
COMMON_RUNTIME_IMAGE_NAME = common-runtime:latest

TOOLCHAIN = stable
NODE_VERSION = lts/*
PLATFORM = ubuntu-latest
FEATURES =

RUST_BACKTRACE=1

# set WireIt variables
WIREIT_LOGGER=metrics
WIREIT_CACHE=none

# Enable Docker BuildKit
DOCKER_BUILDKIT=1

# Enable Docker Compose BuildKit
COMPOSE_DOCKER_CLI_BUILD=1

# .PHONY is a special target that tells make that the listed targets are not files
.PHONY: docker docker-base

docker: docker-base

# Build the base image
docker-base:
	docker build -f Dockerfile -t $(BASE_IMAGE_NAME) . --progress=plain --cache-from=$(BASE_IMAGE_NAME)
# get the digest and assign to Variable
	BASE_IMAGE_DIGEST=$(shell docker inspect --format='{{index .RepoDigests 0}}' $(BASE_IMAGE_NAME))
# tag with digest
	docker tag $(BASE_IMAGE_NAME) $(BASE_IMAGE_DIGEST)
