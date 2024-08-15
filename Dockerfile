# Base image with specific version
ARG BASE_IMAGE=debian:bookworm-slim
FROM ${BASE_IMAGE} AS base

RUN \
  apt update && \
  apt install -y \
  build-essential \
  ca-certificates \
  curl \
  libprotobuf-dev \
  libssl-dev \
  make \
  pkg-config \
  protobuf-compiler \
  tree \
  && rm -rf /var/lib/apt/lists/*

# Rust dependencies
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
  . $HOME/.cargo/env && \
  rustc --version

# Node.js dependencies
ARG NODE_VERSION=lts

RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_VERSION}.x | bash - && \
  apt install -y nodejs && \
  node -v && npm -v

# Make sure Rust and Node.js are in the PATH
ENV PATH=$PATH:/root/.cargo/bin

# Verify Rust and Node.js installations
RUN node -v
RUN rustc -V

# Install Rust tools
RUN . $HOME/.cargo/env && \
  cargo install cargo-binstall && \
  cargo binstall \
  cargo-component \
  cargo-nextest \
  wit-deps-cli \
  wasm-tools \
  --no-confirm

# Install global npm packages
RUN npm install -g \
  wireit \
  @bytecodealliance/jco \
  @bytecodealliance/componentize-js
