FROM debian:latest AS rust-node-base

# Define arguments for the versions of Rust and Node.js, defaulting to 'latest'
ARG RUST_VERSION=latest
ARG NODE_VERSION=latest

# Install dependencies
RUN apt update && apt install -y \
  build-essential \
  ca-certificates \
  curl \
  libprotobuf-dev \
  libssl-dev \
  make \
  pkg-config \
  protobuf-compiler \
  tree \
  && \
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

FROM rust-node-base AS rust-node-builder

# Add rust target and cargo tools
RUN . $HOME/.cargo/env && \
  cargo install cargo-binstall && \
  cargo binstall cargo-component --no-confirm && \
  cargo binstall cargo-nextest --no-confirm && \
  cargo binstall wit-deps-cli --no-confirm && \
  cargo binstall wasm-tools --no-confirm

# install npm install -g wireit
RUN npm install -g \
  wireit \
  @bytecodealliance/jco \
  @bytecodealliance/componentize-js

WORKDIR /build-root

COPY . .

RUN ls -alvh
RUN tree

# Set the working directory to /build-root/typescript and install npm dependencies
WORKDIR /build-root/typescript
RUN npm ci
RUN npm run build


# Set the working directory back to /build-root
WORKDIR /build-root

# run wit-deps for common-javascript-interpreter
WORKDIR /build-root/rust/common-javascript-interpreter
RUN wit-deps

# build common-javascript-interpreter
WORKDIR /build-root
RUN cargo component build -p common-javascript-interpreter --release

# verify wasm file exists
RUN ls -lahv /build-root/target/
RUN ls -lahv /build-root/target/release/
RUN ls -lahv /build-root/target/wasm32-wasip1/release/

ENV COMMON_JAVASCRIPT_INTERPRETER_WASM_PATH=/build-root/target/wasm32-wasip1/release/common_javascript_interpreter.wasm

RUN test -f $COMMON_JAVASCRIPT_INTERPRETER_WASM_PATH

# perform rust build
WORKDIR /build-root
RUN cargo build --release

# list outputs
RUN ls -alvh /build-root/target/release
