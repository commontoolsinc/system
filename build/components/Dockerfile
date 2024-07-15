FROM rust:1.79 as builder

ENV NODE_VERSION=20.14.0
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
ENV NVM_DIR=/root/.nvm
RUN . "$NVM_DIR/nvm.sh" && nvm install ${NODE_VERSION} 
RUN . "$NVM_DIR/nvm.sh" && nvm use v${NODE_VERSION}
RUN . "$NVM_DIR/nvm.sh" && nvm alias default v${NODE_VERSION}
ENV PATH="$PATH:/root/.nvm/versions/node/v${NODE_VERSION}/bin/"

RUN npm install -g @bytecodealliance/jco @bytecodealliance/componentize-js

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-component wasm-tools wit-deps-cli --no-confirm --force
RUN rustup target add wasm32-wasi
RUN rustup component add rustfmt

WORKDIR /build-root

COPY "./Cargo.toml" "./Cargo.lock" ./
COPY ./rust ./rust
COPY ./typescript ./typescript

WORKDIR /build-root

RUN cargo component build -p common-javascript-interpreter --release

WORKDIR /host
VOLUME /host

