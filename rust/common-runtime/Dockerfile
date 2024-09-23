FROM debian:bookworm-slim AS base

FROM nixos/nix

RUN nix-channel --update

WORKDIR /build_root

COPY . /build_root

RUN ls -lah

RUN nix \
    --extra-experimental-features "nix-command flakes" \
    --option filter-syscalls false \
    build

