{
  description = "Child flake";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    common-ancestor.url = "path:../../../.";
  };

  outputs = { nixpkgs, flake-utils, common-ancestor, ... }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          common = common-ancestor.packages.${system};
        in
        {
          devShells = {
            default = pkgs.mkShell {
              buildInputs = with pkgs; [
                nodejs
                common.runtime-npm-package
              ];
              shellHook = ''
                npm pkg set dependencies.common-runtime="file:${common.runtime-npm-package}/common-runtime"
              '';
            };
          };
        }
      );
}
