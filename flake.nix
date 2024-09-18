/**
 * Common Tools System
 *
 * This file constitutes a software development environment that
 * can reliably build the Common Tools System.
 * 
 * In order to use it, you need Nix with the "flakes" feature enabled.
 * Follow the instructions at https://zero-to-nix.com/start/install for
 * a quick start to get Nix set up on your local system.
 *
 * After you have Nix set up, you can launch a development environment
 * in your terminal by typing `nix develop`. If you use a specialty shell
 * like `zsh`, you will want to add an extra argument: `nix develop -c zsh`.
 *
 * If you want to use the Rust nightly toolchain in your development
 * environment, you can switch to it using `nix develop .#nightly`.
 */

{
  description = "Common Tools System";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          inherit (pkgs) lib stdenv darwin;

          common-build-inputs = with pkgs; [
            openssl
            pkg-config
            protobuf
            wasm-pack
            cargo-component
            cargo-nextest
            wasm-bindgen-cli
            wit-deps-cli
            nodejs
            jco
          ] ++ lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.SystemConfiguration
            darwin.apple_sdk.frameworks.Security
          ];

          /**
           * Helper function to select a Rust toolchain by name e.g., "stable", "nightly".
           * The resulting derivation includes Cargo Wasm32 targets.
           */
          rustToolchain = toolchain:
            let
              rustToolchain = pkgs.rust-bin.${toolchain}.latest.default.override {
                targets = [
                  "aarch64-apple-darwin"
                  "wasm32-unknown-unknown"
                  "wasm32-wasip1"
                ];
              };
            in
            if builtins.hasAttr toolchain pkgs.rust-bin then
              rustToolchain
            else
              throw "Unsupported Rust toolchain: ${toolchain}";

          /**
           * Helper function to template a dev shell for different
           * Rust toolchains
           */
          makeDevShell = toolchain:
            let
              rust-toolchain = rustToolchain toolchain;
            in
            with pkgs; mkShell {
              buildInputs = [
                rust-toolchain
                chromium
                chromedriver
                google-cloud-sdk
              ] ++ common-build-inputs;

              shellHook = ''
                bash ./wit/wit-tools.sh deps
              '';
            };

          /**
           * Re-export jco from our fake root package; we do this because
           * @bytecodealliance/jco is a non-trivial package to build from
           * scratch (it entails Rust compilation, Wasm artifact generation
           * and JavaScript transpiling / bundling), and there is no practical
           * path with Nix to do the equivalent of `npm install -g`.
           *
           * NOTE: When updating the NPM dependencies, you need to refresh the
           * `npmDepsHash`. The way to do this is to set the value to
           * `pkgs.lib.fakeHash`, then run a build or devshell, then copy the
           * "expected" value from the failed run and replace `fakeHash` with it.
           */
          jco = pkgs.buildNpmPackage {
            name = "jco";
            src = ./typescript;
            dontNpmBuild = true;
            npmDepsHash = "sha256-Nfhe2YyD7fhHoOiQEnrb8C6A0RfgOMa1xFqbTIAykyA=";
          };

          wit-deps-cli = pkgs.rustPlatform.buildRustPackage rec {
            pname = "wit-deps-cli";
            version = "0.3.4";
            buildInputs = [ pkgs.rust-bin.stable.latest.default ] ++ lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.SystemConfiguration
              darwin.apple_sdk.frameworks.Security
            ];

            src = pkgs.fetchCrate {
              inherit pname version;
              sha256 = "sha256-TMECk5cIZOXgrUQlEPo6P+TPfmgVgO2Mf7phkR4Qw/U=";
            };

            cargoHash = "sha256-+7eHg3bQIt2ZhQCP0p0hGnn+yz9NX+1n45Yr5cZmoUA=";
          };

          wasm-bindgen-cli = pkgs.rustPlatform.buildRustPackage
            rec {
              pname = "wasm-bindgen-cli";
              # NOTE: Version must be kept in sync with Cargo.toml
              # version of `wasm-bindgen` dependency!
              version = "0.2.93";
              buildInputs = [ pkgs.rust-bin.stable.latest.default ] ++ lib.optionals stdenv.isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
                darwin.apple_sdk.frameworks.Security
              ];

              src = pkgs.fetchCrate {
                inherit pname version;
                sha256 = "sha256-DDdu5mM3gneraM85pAepBXWn3TMofarVR4NbjMdz3r0=";
              };

              cargoHash = "sha256-birrg+XABBHHKJxfTKAMSlmTVYLmnmqMDfRnmG6g/YQ=";
            };
        in
        {
          packages = rec {
            builder =
              let
                rust-toolchain = rustToolchain "stable";
                rust-platform = pkgs.makeRustPlatform {
                  cargo = rust-toolchain;
                  rustc = rust-toolchain;
                };
              in
              rust-platform.buildRustPackage {
                name = "builder";
                src = ./.;
                /* Don't run tests as part of this task */
                doCheck = false;
                buildPhase = ''
                  bash ./wit/wit-tools.sh deps
                  cargo build -p common-builder --release
                '';
                installPhase = ''
                  mkdir -p $out/builder
                  cp ./target/release/builder $out/builder
                '';

                nativeBuildInputs = [ rust-toolchain ] ++ common-build-inputs;
                cargoLock = {
                  lockFile = ./Cargo.lock;
                };
              };

            runtime =
              let
                rust-toolchain = rustToolchain "stable";
                rust-platform = pkgs.makeRustPlatform {
                  cargo = rust-toolchain;
                  rustc = rust-toolchain;
                };
              in
              rust-platform.buildRustPackage {
                name = "runtime";
                src = ./.;
                /* Don't run tests as part of this task */
                doCheck = false;
                buildPhase = ''
                  bash ./wit/wit-tools.sh deps
                  cargo build -p common-runtime --release
                '';
                installPhase = ''
                  mkdir -p $out/runtime
                  cp ./target/release/runtime $out/runtime
                '';

                nativeBuildInputs = [ rust-toolchain ] ++ common-build-inputs;
                cargoLock = {
                  lockFile = ./Cargo.lock;
                };
              };

            web-runtime =
              let
                rust-toolchain = rustToolchain "stable";
                rust-platform = pkgs.makeRustPlatform {
                  cargo = rust-toolchain;
                  rustc = rust-toolchain;
                };
              in
              rust-platform.buildRustPackage {
                name = "web-runtime";
                src = ./.;
                /* Don't run tests as part of this task */
                doCheck = false;
                buildPhase = ''
                  # NOTE: wasm-pack currently requires a writable $HOME
                  # directory to be set
                  # SEE: https://github.com/rustwasm/wasm-pack/issues/1318#issuecomment-1713377536
                  export HOME=`pwd`

                  bash ./wit/wit-tools.sh deps
                  wasm-pack build --target web -m no-install ./rust/common-runtime
                '';
                installPhase = ''
                  mkdir -p $out
                  mv ./rust/common-runtime/pkg $out/common-runtime
                '';

                nativeBuildInputs = [ rust-toolchain ] ++ common-build-inputs;
                cargoLock = {
                  lockFile = ./Cargo.lock;
                };
              };

            runtime-docker-image = pkgs.dockerTools.buildLayeredImage {
              name = "common-runtime";
              tag = "latest";
              created = "now";
              config.Entrypoint = [ "${runtime}/runtime/runtime" ];
            };

            builder-docker-image = pkgs.dockerTools.buildLayeredImage {
              name = "common-builder";
              tag = "latest";
              created = "now";
              contents = [
                jco
              ];
              # NOTE: This is needed because the extremely minimal base image
              # doesn't have a /tmp! And, for now we initialize a temporary DB
              # for caching build artifacts.
              fakeRootCommands = ''
                mkdir -p /tmp
              '';
              enableFakechroot = true;
              config.Entrypoint = [ "${builder}/builder/builder" ];
            };
          };

          devShells = {
            default = makeDevShell "stable";
            nightly = makeDevShell "nightly";
          };
        }
      );
}
