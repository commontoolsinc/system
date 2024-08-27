# Development

Notes on building and testing `system`.

This document might get out of date - check github workflow files (`.github/workflows/rust.yaml` et al)
for the most up-to-date build that are run as part of the CI process.

## Dependencies

This project uses both Rust and Node.js. Both toolchains and dependencies will need to be installed.

### System

Install [protobuf-compiler] for your system.

### Rust

First, install [cargo] and [binstall] to set up the toolchain.
Then, install some Rust tools:

```bash
cargo binstall wit-deps-cli wasm-tools cargo-component cargo-nextest
```

### Node.js

First, install [node.js].
Then, install some node.js tools:

```bash
npm install -g @bytecodealliance/jco @bytecodealliance/componentize-js
```

### WebAssembly Interface Types (WIT) dependencies

Finally, install the WIT dependencies:

```bash
./wit/wit-tools.sh deps
```

## Tests

Once all dependencies are set up, run:

```bash
cargo test
```

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
