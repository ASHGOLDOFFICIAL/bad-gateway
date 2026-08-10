{
  description = "bad-gateway dev environment";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.minimal.override {
          extensions = [
            "clippy"
            "rust-analyzer"
            "rust-src"
          ];
        };

        nightlyRustfmt = pkgs.rust-bin.nightly.latest.rustfmt;
      in {
        devShell = pkgs.mkShell {
          nativeBuildInputs = [ rustToolchain nightlyRustfmt ];
          buildInputs = with pkgs; [ pkg-config ];
        };
      }
    );
}
        
