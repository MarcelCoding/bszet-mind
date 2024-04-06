{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = (import nixpkgs) {
            inherit system;
          };
        in
        {
          packages = rec {
            bszet-mind = pkgs.callPackage ./derivation.nix {
              cargoToml = ./bszet-mind/Cargo.toml;
            };
            default = bszet-mind;
          };
        }
      ) // {
      overlays.default = _: prev: {
        bszet-mind = self.packages."${prev.system}".bszet-mind;
      };

      nixosModules = rec {
        bszet-mind = import ./module.nix;
        default = bszet-mind;
      };
    };
}
