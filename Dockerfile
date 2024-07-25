ARG BASE_IMAGE=debian:latest
FROM ${BASE_IMAGE} AS rust-node-base

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

# Add rust target and cargo tools
RUN . $HOME/.cargo/env && \
  cargo install cargo-binstall && \
  cargo binstall \
  cargo-component \
  cargo-nextest \
  wit-deps-cli \
  wasm-tools \
  --no-confirm

# install npm install -g wireit
RUN npm install -g \
  wireit \
  @bytecodealliance/jco \
  @bytecodealliance/componentize-js
