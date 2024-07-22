FROM debian:latest as rust-node-builder

# Define arguments for the versions of Rust and Node.js, defaulting to 'latest'
ARG RUST_VERSION=latest
ARG NODE_VERSION=latest

# Install dependencies
RUN apt update && apt install -y \
  curl \
  build-essential \
  ca-certificates && \
  rm -rf /var/lib/apt/lists/*

# Install Rust
RUN if [ "$RUST_VERSION" = "latest" ]; then \
  curl https://sh.rustup.rs -sSf | sh -s -- -y; \
  else \
  curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}; \
  fi && \
  . $HOME/.cargo/env && \
  rustc --version

# Install Node.js
RUN if [ "$NODE_VERSION" = "latest" ]; then \
  curl -sL https://deb.nodesource.com/setup_current.x | bash -; \
  else \
  curl -sL https://deb.nodesource.com/setup_${NODE_VERSION}.x | bash -; \
  fi && \
  apt-get install -y nodejs && \
  node -v && \
  npm -v

# Make sure Rust and Node.js are in the PATH
ENV PATH=$PATH:/root/.cargo/bin

# Verify installations
RUN node -v
RUN rustc -V

FROM rust-node-builder as rust-node-wasm-builder

RUN apt-get update && apt-get install -y \
  curl \
  build-essential \
  pkg-config \
  libssl-dev \
  protobuf-compiler \
  libprotobuf-dev && \
  rm -rf /var/lib/apt/lists/*

# Add rust target and cargo tools
RUN . $HOME/.cargo/env && \
  rustup target add wasm32-unknown-unknown && \
  cargo install cargo-binstall && \
  cargo binstall cargo-component --no-confirm && \
  cargo binstall cargo-nextest --no-confirm && \
  cargo binstall wit-deps-cli --no-confirm && \
  cargo binstall wasm-tools --no-confirm

WORKDIR /build-root

COPY . .

WORKDIR /build-root/typescript

RUN npm ci
RUN npm run build

# Set the working directory to /build-root
WORKDIR /build-root

# Build the common-javascript-interpreter to generate the wasm file
RUN cargo build --release -p common-javascript-interpreter

# Expose the path to the wasm file as an environment variable
ENV COMMON_JAVASCRIPT_INTERPRETER_WASM_PATH=/build-root/target/release/deps/common_javascript_interpreter.wasm


# Set the build context back to the root for subsequent builds
WORKDIR /build-root
