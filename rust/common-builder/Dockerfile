# This Dockerfile sets up a multi-stage build for a Rust project with TypeScript dependencies.
# It first sets up a builder stage using the rust:1.76 image.
# It installs Node.js using NVM and sets the desired version.
# Then, it copies the project files and installs the necessary dependencies for both Rust and TypeScript.
# Next, it builds the Rust project and creates a release binary.
# In the second stage, it uses the node:latest image to create the final runtime image.
# It installs additional dependencies and copies the release binary from the builder stage.
# Finally, it sets environment variables and specifies the entrypoint for the runtime image.

FROM rust:1.76 as builder

# Set the desired Node.js version using NVM
ENV NODE_VERSION=20.13.1
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
ENV NVM_DIR=/root/.nvm
RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm use v${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm alias default v${NODE_VERSION}
ENV PATH="/root/.nvm/versions/node/v${NODE_VERSION}/bin/:${PATH}"

WORKDIR /build-root

# Copy project files
COPY "./Cargo.toml" "./Cargo.lock" ./
COPY ./rust ./rust
COPY ./typescript ./typescript

# Install Rust and TypeScript dependencies
RUN rustup target add wasm32-wasi
RUN cargo install wasm-tools wit-deps-cli

WORKDIR /build-root/typescript

# Install TypeScript dependencies and build
RUN npm install -g @bytecodealliance/jco
RUN npm ci
RUN npm run build

# Set the working directory to /build-root
WORKDIR /build-root

# Build the Rust project
RUN cargo build --release --bin runtime

FROM node:latest

WORKDIR /runtime

EXPOSE 8080

# Install additional dependencies
RUN apt-get update && apt-get install -y libssl-dev ca-certificates
RUN npm install -g @bytecodealliance/componentize-js
RUN npm install -g @bytecodealliance/jco

# Copy the release binary from the builder stage
COPY --from=builder /build-root/target/release/runtime /usr/bin/runtime

# Set environment variables
ENV RUST_LOG="debug"
ENV UPSTREAM="localhost:5173"

# Specify the entrypoint for the runtime image
ENTRYPOINT ["/usr/bin/runtime"]
