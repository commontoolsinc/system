{
  lib,
  config,
  dream2nix,
  ...
}: {
  imports = [
    dream2nix.modules.dream2nix.nodejs-package-lock-v3
    dream2nix.modules.dream2nix.nodejs-granular-v3
  ];

  mkDerivation = {
    src = ./.;
    nativeBuildInputs = [
      config.deps.wit-deps-cli
      config.deps.wasm-tools
      config.deps.nodejs
      config.deps.nodePackages.npm
    ];
    
    # Add a preBuild hook to ensure wit-deps is available
    preBuild = ''
      export PATH="${config.deps.wasm-tools}/bin:${config.deps.wit-deps-cli}/bin:$NIX_BUILD_TOP/package/@commmontools/.bin:$PATH"

      # Configure npm to work offline and use the local node_modules
      # npm config set offline true
      # npm config set prefer-offline true
      
      # Set NODE_PATH to include the local node_modules
      # export NODE_PATH="$PWD/node_modules:$NODE_PATH"
      
      # npx @bytecodealliance/jco --help
      # # Create a wrapper for npx that forces offline mode
      # cat > npx-offline <<EOF
      # #!/bin/sh
      # npx --offline --no-install "\$@"
      # EOF
      # chmod +x npx-offline
      
      # # Add the wrapper to the PATH
      # export PATH="$PWD:$PATH"
    '';
  };

  deps = {nixpkgs, ...}: {
    inherit
      (nixpkgs)
      fetchFromGitHub
      stdenv
      ;
  };

  nodejs-package-lock-v3 = {
    packageLockFile = "${config.mkDerivation.src}/package-lock.json";
  };

  name = "@commmontools/libs";
  version = "1.5.0";
}
