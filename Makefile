# Variables
BASE_IMAGE_NAME=common-tools/base:latest
COMMON_BUILDER_IMAGE_NAME=common-builder:latest
COMMON_RUNTIME_IMAGE_NAME=common-runtime:latest

# .PHONY is a special target that tells make that the listed targets are not files
.PHONY: all base common-builder common-runtime

all: base common-builder common-runtime

# Build the base image
base:
	docker build -f Dockerfile -t $(BASE_IMAGE_NAME) . --progress=plain
	# get the digest and assign to Variable
	BASE_IMAGE_DIGEST=$(shell docker inspect --format='{{index .RepoDigests 0}}' $(BASE_IMAGE_NAME))
	# tag with digest
	docker tag $(BASE_IMAGE_NAME) $(BASE_IMAGE_DIGEST)

# Build the common-builder image
common-builder: base
	docker build --build-arg BASE_IMAGE=$(BASE_IMAGE_NAME) -f rust/common-builder/Dockerfile -t $(COMMON_BUILDER_IMAGE_NAME) .

# Build the common-runtime image
common-runtime: base
	docker build --build-arg BASE_IMAGE=$(BASE_IMAGE_NAME) -f rust/common-runtime/Dockerfile -t $(COMMON_RUNTIME_IMAGE_NAME) .

# Clean up the images
clean:
	docker rmi $(COMMON_BUILDER_IMAGE_NAME) $(COMMON_RUNTIME_IMAGE_NAME) $(BASE_IMAGE_NAME)
