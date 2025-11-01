{
  description = "A Homebrew-like CLI for Nix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = import nixpkgs { inherit system; };
      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "nixbrew";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
        
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/nixbrew";
        };
      });
}