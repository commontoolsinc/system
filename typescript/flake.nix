{
  # This example flake.nix is pretty generic and the same for all
  # examples, except when they define devShells or extra packages.
  description = "Dream2nix example flake";

  # We import the latest commit of dream2nix main branch and instruct nix to
  # re-use the nixpkgs revision referenced by dream2nix.
  # This is what we test in CI with, but you can generally refer to any
  # recent nixpkgs commit here.
  inputs = {
    dream2nix.url = "github:nix-community/dream2nix";
    nixpkgs.follows = "dream2nix/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    dream2nix,
    nixpkgs,
    rust-overlay
  }: let
    # A helper that helps us define the attributes below for
    # all systems we care about.
    eachSystem = nixpkgs.lib.genAttrs [
      "aarch64-darwin"
      "aarch64-linux"
      "x86_64-darwin"
      "x86_64-linux"
    ];
  in {
    packages = eachSystem (system: let 
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      
      rust = pkgs.rust-bin.stable.latest.default;

      wit-deps-cli = pkgs.rustPlatform.buildRustPackage rec {
        pname = "wit-deps-cli";
        version = "0.3.4";
        buildInputs = [ rust ];

        src = pkgs.fetchCrate {
          inherit pname version;
          sha256 = "sha256-TMECk5cIZOXgrUQlEPo6P+TPfmgVgO2Mf7phkR4Qw/U=";
        };

        cargoSha256 = "sha256-+7eHg3bQIt2ZhQCP0p0hGnn+yz9NX+1n45Yr5cZmoUA=";
      }; 

      wasm-tools = pkgs.rustPlatform.buildRustPackage rec {
        pname = "wasm-tools";
        version = "1.215.0";
        buildInputs = [ rust ];

        src = pkgs.fetchCrate {
          inherit pname version;
          sha256 = "sha256-TGs7uOi6L7oZcWtuRsRviYdzfICh93lnAt30VvcN8jw=";
        };
        fetchSubmodules = true;
        # TODO: The 'check' runs `cargo test` but this fails on
        # https://github.com/bytecodealliance/wasm-tools/blob/c26e7d18e93cd65fd50a1f85b330bb3fa315ad15/tests/roundtrip.rs#L85
        doCheck = false;

        cargoSha256 = "sha256-X5iw22ZZUeHRQiUx3Y2AQ+1nMc1IyyHJtfLoYCrgrso=";
      }; 
    
    in {


      # For each system, we define our default package
      # by passing in our desired nixpkgs revision plus
      # any dream2nix modules needed by it.
      default = dream2nix.lib.evalModules {
        packageSets.nixpkgs = nixpkgs.legacyPackages.${system};

        modules = [
          # Import our actual package definiton as a dream2nix module from ./default.nix
          ./default.nix
          {
            # Aid dream2nix to find the project root. This setup should also works for mono
            # repos. If you only have a single project, the defaults should be good enough.
            paths.projectRoot = ./.;
            # can be changed to ".git" or "flake.nix" to get rid of .project-root
            paths.projectRootFile = "flake.nix";
            paths.package = ./.;
          }
          {
            deps = {
              inherit wit-deps-cli wasm-tools;
              inherit (nixpkgs.legacyPackages.${system}) nodejs nodePackages;
            };
            # deps = {
            #   wit-deps-cli = {
            #     rustPackage = wit-deps-cli;
            #     runtime = true;
            #   };
            # };
          }
        ];
      };
      inherit wit-deps-cli;
    });

    devShells = eachSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      default = pkgs.mkShell {
        buildInputs = [
          self.packages.${system}.wit-deps-cli
          # Add other development tools here
        ];
      };
    });

  };
}
