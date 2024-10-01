# Development

Notes on building and testing `system`.

This document might get out of date - check github workflow files (`.github/workflows/rust.yaml` et al)
for the most up-to-date build that are run as part of the CI process.

## Nix 

This project uses Nix to setup the dependencies needed, including Rust and Node.js. Nix is used to produce build targets, as well as scaffolding a shell for development. All build targets and devshells are configured in `flake.nix`.

It's recommended to install nix via instructions on [zero-to-nix](https://zero-to-nix.com/start/install), which also configures flakes.

To use the nix devshell that includes all required dependencies:

```sh
nix develop
```

For non-`bash` shells, a `-c` flag can be provided:

```sh
nix develop -c zsh
```

For alternate environments (other than default, only `nightly` supported), try:

```sh
nix develop .#nightly
```

After you drop into the Nix development environment, you can
build the project as normal e.g., `cargo build`, `cargo test`.

## Targets

Currently there are several build targets defined in the `flake.nix`. All flake actions can be displayed via `nix flake show`. To build a target, execute the following replacing `TARGET` with the build target name:

```sh
nix build .#TARGET
```

The output of the builds can be found in `./result`.

* `runtime`: The runtime server, produced by [common-runtime]. 
* `builder`: The builder server, produced by [common-builder].
* `runtime-web`: The JavaScript module wrapping a wasm32 build of [common-runtime].
* `runtime-image`: A Docker container image of [common-runtime].
* `builder-image`: A Docker container image of [common-builder].
* `wasm-components`: Wasm artifacts used by the runtime. Used for optimizing e.g. CI builds.

## Testing

See `./scripts/ops-test` for running unit, integration, doc, and lint tests. Mostly a wrapper for running cargo tests.

```sh
# Run unit tests
nix develop .
./scripts/ops-test unit
```

## WebAssembly Interface Types (WIT) dependencies

WIT definitions are used to generate bindings in the project. If building via nix, this is handled. If developing and WIT definitions are modified, then this command must be run:

```bash
./wit/wit-tools.sh deps
```

## Running Services

The primary service is the [common-runtime]. Currently, an additional service is needed to run alongside the runtime, the [common-builder] service, which compiles the wasm artifacts on demand for Compiled Modules, and handles bundling of artifacts for Interpreted Modules.

Generally the [common-runtime] uses port 8081, and [common-builder] port 8082.

### Running via cargo

To run these services via cargo, ensure your environment is setup with nix, run these two services in separate shells:

```
RUST_LOG=debug cargo run -p common-builder
```

```
RUST_LOG=debug cargo run -p common-runtime -- --builder-address 127.0.0.1:8082
```

The runtime can be accessed via `127.0.0.1:8081`.

### Running via Docker

> :warning: Currently resolving an issue with docker compose [#210], run via cargo in the interim.

First, install docker for your platform. The images must first be built and loaded into the docker environment.

First, build the runtime and load into docker:

```sh
nix build .#runtime-docker-image
docker load < result
```

The `common-runtime:latest` image should be added to your docker environment:

```sh
$ docker image ls
REPOSITORY       TAG       IMAGE ID       CREATED          SIZE
common-runtime   latest    0c2736dcb30c   2 minutes ago   57.4MB
```

Now build and load the builder service image:

```sh
nix build .#builder-docker-image
docker load < result
```

Both images should be in the docker environment:

```sh
$ docker image ls
REPOSITORY       TAG       IMAGE ID       CREATED          SIZE
common-builder   latest    4a0ee08316cf   44 seconds ago   519MB
common-runtime   latest    0c2736dcb30c   2 minutes ago    57.4MB
```

With images built, docker compose can now be run, exposing these services:

```sh
docker compose up
```

#### Helpful Docker Commands


* Stop all containers: `docker stop $(docker ps -a -q)`
* Remove all containers: `docker rm $(docker ps -a -q)`

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
[common-runtime]: ./rust/common-runtime
[common-builder]: ./rust/common-builder

## Makefile Overview

This `Makefile` coordinates the build process for a Rust and TypeScript project using Docker. It integrates various development tools and dependencies for efficient multi-environment builds. Below are key targets and variables.

### Usage

- **Default build**:

```sh
make
```

- **Build all Docker images**:

```sh
make docker
```

- **Local build**:

```sh
make local
```

- **Force a build step**:

```sh
make -B <target>
```

### Variables

- **`BASE_IMAGE_NAME`**: Base Docker image name.
- **`COMMON_MONOLITH_IMAGE_NAME`**: Name for the monolithic Docker image.
- **`COMMON_BUILDER_IMAGE_NAME`**: Builder Docker image name.
- **`COMMON_RUNTIME_IMAGE_NAME`**: Runtime Docker image name.
- **`RUNTIME_BASE_IMAGE`**: Base image for the runtime environment (default: Debian latest).
- **`TOOLCHAIN`**: Rust toolchain version.
- **`NODE_VERSION`**: Node.js version (default: latest LTS).
- **`PLATFORM`**: Build platform (default: Ubuntu latest).
- **`RUST_BACKTRACE`**: Enables Rust backtraces.
- **`WIREIT_LOGGER`**: Configures WireIt logging (default: metrics).
- **`DOCKER_BUILDKIT`**: Enables Docker BuildKit.
- **`COMPOSE_DOCKER_CLI_BUILD`**: Enables BuildKit for Docker Compose.

### Targets

#### `all`

- **Description**: Default target that triggers the local build process.
- **Dependencies**: `local`

#### `local`

- **Description**: Builds the Rust project and lists the build outputs.
- **Dependencies**: `build_rust`, `list_outputs`

#### `docker`

- **Description**: Builds all Docker images.
- **Dependencies**: `docker_base`, `docker_monolith`, `docker_builder`, `docker_runtime`

#### `docker_base`

- **Description**: Builds the base Docker image.
- **Dependencies**: `.docker_base.done`

#### `docker_monolith`

- **Description**: Builds the monolithic Docker image.
- **Dependencies**: `.docker_monolith.done`, `docker_base`

#### `docker_builder`

- **Description**: Builds the Docker image for the build process.
- **Dependencies**: `.docker_builder.done`, `docker_base`

#### `docker_runtime`

- **Description**: Builds the Docker image for the runtime environment.
- **Dependencies**: `.docker_runtime.done`, `docker_base`

#### `build_typescript`

- **Description**: Installs npm dependencies and builds the TypeScript project.
- **Dependencies**: `.build_typescript.done`

#### `build_rust`

- **Description**: Builds the Rust project in release mode.
- **Dependencies**: `.build_rust.done`

#### `list_outputs`

- **Description**: Lists the outputs of the Rust build.
- **Dependencies**: `.build_rust.done`

### Phony Targets

These targets do not correspond to files and are always executed when invoked.
The `.done` files are used to track the completion of build steps.

- `all`
- `local`
- `docker`
- `docker_base`
- `docker_monolith`
- `docker_builder`
- `docker_runtime`
- `build_typescript`
- `build_rust`
- `list_outputs`

### The `.done` Files

`.done` files mark the completion of build steps, preventing redundant executions and enhancing efficiency. Each target checks for its `.done` file before execution, creating it upon successful completion.

## Docker Layer Caching

Optimise Docker builds by structuring the `Dockerfile` and `Makefile` to utilise layer caching effectively. Place infrequently changing commands early, use multiple `RUN` instructions, and leverage `.done` files to track completed steps. This approach speeds up builds and maintains cache effectiveness.

## Purpose and Usage of Dockerfiles with the Makefile

### Base Dockerfile

- **Name**: `Dockerfile`
- **Purpose**: Sets up the base image with development tools for Rust and Node.js.

### Rust/Node Dockerfile

- **Name**: `rust/Dockerfile`
- **Purpose**: Builds the project and creates runtime images.
- **Key Steps**:
  - **Stage 1**: Copies project files and builds the project.
  - **Stage 2**: Prepares the runtime environment with the compiled binaries.

### Integration with the Makefile targets

1. **`docker_base`**: Builds the base image.
2. **`docker_monolith`**: Compiles the project into a monolithic image.
3. **`docker_builder`**: Creates a build-optimised image.
4. **`docker_runtime`**: Builds the runtime environment image.

### Workflow Summary

1. **Build Base Image**: `docker_base`
2. **Compile Project**: `docker_monolith`
3. **Specialised Builds**: `docker_builder`, `docker_runtime`
4. **Incremental Builds**: Utilise `.done` files for efficiency.

This setup ensures efficient and modular builds for development and production environments.
