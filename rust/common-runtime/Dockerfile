# rust/common-runtime/Dockerfile
ARG BASE_IMAGE
FROM ${BASE_IMAGE} AS builder

# Set the working directory
WORKDIR /build-root

# Ensure the wasm file is available
COPY --from=builder ${COMMON_JAVASCRIPT_INTERPRETER_WASM_PATH} /usr/local/bin/common_javascript_interpreter.wasm
ENV COMMON_JAVASCRIPT_INTERPRETER_WASM_PATH /usr/local/bin/common_javascript_interpreter.wasm

# Build the Rust project
RUN cargo build --release -p common-runtime

# Create the final image
FROM debian:buster-slim

# Install any necessary runtime dependencies
RUN apt-get update && apt-get install -y \
  libssl1.1 \
  ca-certificates && \
  rm -rf /var/lib/apt/lists/*

# Copy the built application from the builder stage
COPY --from=builder /build-root/target/release/common-runtime /usr/local/bin/common-runtime
COPY --from=builder /usr/local/bin/common_javascript_interpreter.wasm /usr/local/bin/common_javascript_interpreter.wasm

# Set the entry point for the container
ENTRYPOINT ["common-runtime"]
