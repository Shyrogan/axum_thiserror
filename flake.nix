{
  description = "axum_thiserror — a procedural macro associating HTTP status codes with error types";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
            "rustfmt"
          ];
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };

        manifest = (pkgs.lib.importTOML ./Cargo.toml).package;

        axum_thiserror = rustPlatform.buildRustPackage {
          pname = manifest.name;
          inherit (manifest) version;

          src = pkgs.lib.cleanSource ./.;

          cargoLock.lockFile = ./Cargo.lock;

          meta = {
            inherit (manifest) description;
            homepage = manifest.repository;
            license = pkgs.lib.licenses.mit;
            maintainers = [ ];
          };
        };
      in
      {
        packages = {
          default = axum_thiserror;
          axum_thiserror = axum_thiserror;
        };

        devShells.default = pkgs.mkShell {
          packages = [ rustToolchain ];

          env.RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        };

        checks.default = axum_thiserror;

        formatter = pkgs.nixfmt-rfc-style;
      }
    );
}
