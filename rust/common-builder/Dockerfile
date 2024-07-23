# rust/common-builder/Dockerfile
ARG BASE_IMAGE
FROM ${BASE_IMAGE} AS builder

# Set the working directory
WORKDIR /build-root

# Build the Rust project
RUN cargo build --release --bin common-builder

FROM debian:buster-slim

WORKDIR /common-builder

EXPOSE 8080

# Install additional dependencies
RUN apt-get update && apt-get install -y libssl-dev ca-certificates
RUN npm install -g @bytecodealliance/componentize-js
RUN npm install -g @bytecodealliance/jco

# Copy the release binary from the builder stage
COPY --from=builder /build-root/target/release/common-builder /usr/bin/common-builder

# Set environment variables
ENV RUST_LOG="debug"
ENV UPSTREAM="localhost:5173"

# Specify the entrypoint for the runtime image
ENTRYPOINT ["common-builder"]
