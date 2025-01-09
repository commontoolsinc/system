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
            wasm-bindgen-cli
            wit-deps-cli
            wasm-tools
            wasi-virt 
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
              ] ++ lib.optionals stdenv.isLinux [
                chromium
                chromedriver
              ] ++ common-build-inputs;

              shellHook = ''
                bash ./wit/wit-tools.sh deps
              '';
            };

          wit-deps-cli = pkgs.rustPlatform.buildRustPackage rec {
            pname = "wit-deps-cli";
            version = "0.4.0";
            buildInputs = [ pkgs.rust-bin.stable.latest.default ] ++ lib.optionals stdenv.isDarwin [
              darwin.apple_sdk.frameworks.SystemConfiguration
              darwin.apple_sdk.frameworks.Security
            ];

            src = pkgs.fetchCrate {
              inherit pname version;
              sha256 = "sha256-xU4qShOgneX9B5exkxxv/rx/yVPMflvcXQ/rQBOcBRg=";
            };

            cargoHash = "sha256-kjvbGJpdNk+achZykzSUk3dShYT+EXNlxRlc3N2UHpA=";
          };

          wasi-virt = pkgs.rustPlatform.buildRustPackage rec {
            pname = "wasi-virt";
            version = "0.1.0";
            buildInputs = [ pkgs.rust-bin.stable.latest.default ];

            doCheck = false;

            # Currently targeting wasip1/wasi@0.2.0.
            # Use revision before wasi-virt updates to wasip2.
            src = pkgs.fetchFromGitHub {
              owner = "bytecodealliance";
              repo = pname;
              rev = "b662e419bb7";
              hash = "sha256-y/FF5BKyTSVoknu77CGIUU3l8qOoYrUGATqjxMF1pGg=";
            };

            cargoHash = "sha256-VdeNh/MfQjnTjmbIxScCgHipOJ5huPNG1WHF1uFTaFw=";
          };

          wasm-tools = pkgs.rustPlatform.buildRustPackage
            rec {
              pname = "wasm-tools";
              version = "1.218.0";
              buildInputs = [ pkgs.rust-bin.stable.latest.default ] ++ lib.optionals stdenv.isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
                darwin.apple_sdk.frameworks.Security
              ];

              # wasm-tools tests requires a git submodule. Not sure how to resolve
              # currently, so skip the post build tests.
              doCheck = false;

              src = pkgs.fetchCrate {
                inherit pname version;
                sha256 = "sha256-GURmrbsJxq+jHwQ5ERUVaMKXW4+46p8SCYMN/DxQyOs=";
              };

              cargoHash = "sha256-7qCco057lDBs+gPXlitoFslQrq7n2k5+NsFjBcmjBCU=";
            };

          wasm-bindgen-cli = pkgs.rustPlatform.buildRustPackage
            rec {
              pname = "wasm-bindgen-cli";
              # NOTE: Version must be kept in sync with Cargo.toml
              # version of `wasm-bindgen` dependency!
              version = "0.2.99";
              buildInputs = [ pkgs.rust-bin.stable.latest.default ] ++ lib.optionals stdenv.isDarwin [
                darwin.apple_sdk.frameworks.SystemConfiguration
                darwin.apple_sdk.frameworks.Security
              ];

              src = pkgs.fetchCrate {
                inherit pname version;
                sha256 = "sha256-1AN2E9t/lZhbXdVznhTcniy+7ZzlaEp/gwLEAucs6EA=";
              };

              cargoHash = "sha256-DbwAh8RJtW38LJp+J9Ht8fAROK9OabaJ85D9C/Vkve4=";
            };
        in
        {
          packages = rec {
            engine =
              let
                rust-toolchain = rustToolchain "stable";
                rust-platform = pkgs.makeRustPlatform {
                  cargo = rust-toolchain;
                  rustc = rust-toolchain;
                };
              in
              rust-platform.buildRustPackage {
                name = "engine";
                src = ./.;
                doCheck = false;
                buildPhase = ''
                  bash ./wit/wit-tools.sh deps
                  export CARGO_COMPONENT_CACHE_DIR=.cargo-component-cache
                  cargo build -p ct-engine --release \
                    --no-default-features --features storage
                '';
                installPhase = ''
                  mkdir -p $out
                  cp ./target/release/engine $out/ct-engine
                '';

                nativeBuildInputs = [ rust-toolchain ] ++ common-build-inputs;
                cargoLock = {
                  lockFile = ./Cargo.lock;
                  outputHashes = {
                    # Using our own forks of these
                    "js_wasm_runtime_layer-0.4.0" = "sha256-LHhaCqGQCoV7AZKnDkBPdvd3KBda4UkrSlHrODqEELc=";
                    "wasm_component_layer-0.1.18" = "sha256-S/MY+pkmQ93RxQ70/fWEiXjt5lg0YNlM5IQVoDVb3YI=";
                  };
                };
              };
            
            wasm-components =
              let
                rust-toolchain = rustToolchain "stable";
                rust-platform = pkgs.makeRustPlatform {
                  cargo = rust-toolchain;
                  rustc = rust-toolchain;
                };
              in
              rust-platform.buildRustPackage {
                name = "wasm-components";
                src = ./.;
                /* Don't run tests as part of this task */
                doCheck = false;
                buildPhase = ''
                  bash ./wit/wit-tools.sh deps
                  export CARGO_COMPONENT_CACHE_DIR=.cargo-component-cache
                  bash ./scripts/component-build ct-js-vm ./target
                '';
                installPhase = ''
                  mkdir -p $out/wasm-components
                  cp ./target/wasm32-wasip1/release/virt_ct_js_vm.wasm \
                     $out/wasm-components/virt_ct_js_vm.wasm
                '';

                nativeBuildInputs = [ rust-toolchain ] ++ common-build-inputs;
                cargoLock = {
                  lockFile = ./Cargo.lock;
                  outputHashes = {
                    # Using our own forks of these
                    "js_wasm_runtime_layer-0.4.0" = "sha256-LHhaCqGQCoV7AZKnDkBPdvd3KBda4UkrSlHrODqEELc=";
                    "wasm_component_layer-0.1.18" = "sha256-S/MY+pkmQ93RxQ70/fWEiXjt5lg0YNlM5IQVoDVb3YI=";
                  };
                };
              };

            /**
             * Builds the source files for an NPM package that bundles
             * up the engine (compiled to Web Assembly) and an auto-generated
             * JavaScript library with bindings into the runtime's API.
             */
            engine-web =
              let
                rust-toolchain = rustToolchain "stable";
                rust-platform = pkgs.makeRustPlatform {
                  cargo = rust-toolchain;
                  rustc = rust-toolchain;
                };
              in
              rust-platform.buildRustPackage {
                name = "web-engine";
                src = ./.;
                /* Don't run tests as part of this task */
                doCheck = false;
                buildPhase = ''
                  # NOTE: wasm-pack currently requires a writable $HOME
                  # directory to be set
                  # SEE: https://github.com/rustwasm/wasm-pack/issues/1318#issuecomment-1713377536
                  export HOME=`pwd`

                  bash ./wit/wit-tools.sh deps
                  wasm-pack build --target web -m no-install ./rust/ct-engine \
                    -- --no-default-features --features storage

                  cp ./typescript/ct-engine/README.md ./rust/ct-engine/pkg/README.md
                  cp ./typescript/ct-engine/example.html ./rust/ct-engine/pkg/example.html
                '';
                installPhase = ''
                  mkdir -p $out
                  mv ./rust/ct-engine/pkg $out/ct-engine-web
                '';

                nativeBuildInputs = [ rust-toolchain ] ++ common-build-inputs;
                cargoLock = {
                  lockFile = ./Cargo.lock;
                  outputHashes = {
                    # Using our own forks of these
                    "js_wasm_runtime_layer-0.4.0" = "sha256-LHhaCqGQCoV7AZKnDkBPdvd3KBda4UkrSlHrODqEELc=";
                    "wasm_component_layer-0.1.18" = "sha256-S/MY+pkmQ93RxQ70/fWEiXjt5lg0YNlM5IQVoDVb3YI=";
                  };
                };
              };
          };

          devShells = {
            default = makeDevShell "stable";
            nightly = makeDevShell "nightly";
          };
        }
      );
}
