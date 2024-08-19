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
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;

        };

        # Helper function to select a Rust toolchain by name e.g., "stable", "nightly".
        # The resulting derivation includes Cargo Wasm32 targets.
        rustToolchain = toolchain:
          let
            rustToolchain = pkgs.rust-bin.${toolchain}.latest.default.override {
              targets = [
                "wasm32-unknown-unknown"
                "wasm32-wasip1"
              ];
            };
          in
          if builtins.hasAttr toolchain pkgs.rust-bin then
            rustToolchain
          else
            throw "Unsupported Rust toolchain: ${toolchain}";

        # Helper function to template a dev shell for different
        # Rust toolchains
        makeDevShell = toolchain:
          let
            rust-toolchain = rustToolchain toolchain;
          in
          with pkgs; mkShell {
            buildInputs = [
              openssl
              pkg-config
              protobuf
              cargo-component
              wit-deps-cli
              rust-toolchain
              nodejs
              chromedriver
              jco

            ];

            shellHook = ''
              bash ./wit/wit-tools.sh deps
            '';
          };

        # Re-export jco from our fake root package; we do this because
        # @bytecodealliance/jco is a non-trivial package to build from
        # scratch (it entails Rust compilation, Wasm artifact generation
        # and JavaScript transpiling / bundling), and there is no practical
        # path with Nix to do the equivalent of `npm install -g`.
        jco = pkgs.buildNpmPackage {
          name = "jco";
          src = ./typescript;
          dontNpmBuild = true;
          npmDepsHash = "sha256-SvS92dl/ydYhMBKJPbjNZag2KHXhMPBFID/euNiv33w=";
        };

        wit-deps-cli = pkgs.rustPlatform.buildRustPackage rec {
          pname = "wit-deps-cli";
          version = "0.3.4";
          buildInputs = [ pkgs.rust-bin.stable.latest.default ];

          src = pkgs.fetchCrate {
            inherit pname version;
            sha256 = "sha256-TMECk5cIZOXgrUQlEPo6P+TPfmgVgO2Mf7phkR4Qw/U=";
          };

          cargoHash = "sha256-+7eHg3bQIt2ZhQCP0p0hGnn+yz9NX+1n45Yr5cZmoUA=";
        };
      in
      {
        devShells = {
          default = makeDevShell "stable";
          nightly = makeDevShell "nightly";
        };
      }
    );
}
