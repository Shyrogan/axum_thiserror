{
  perSystem = {pkgs, ...}: let
    rustToolchain = pkgs.rust-bin.stable.latest.default;

    rustPlatform = pkgs.makeRustPlatform {
      cargo = rustToolchain;
      rustc = rustToolchain;
    };

    manifest = (pkgs.lib.importTOML ../Cargo.toml).package;

    axum_thiserror = rustPlatform.buildRustPackage {
      pname = manifest.name;
      inherit (manifest) version;

      src = pkgs.lib.cleanSource ../.;

      cargoLock.lockFile = ../Cargo.lock;

      meta = {
        inherit (manifest) description;
        homepage = manifest.repository;
        license = pkgs.lib.licenses.mit;
        maintainers = [];
      };
    };
  in {
    packages = {
      default = axum_thiserror;
      axum_thiserror = axum_thiserror;
    };

    checks.default = axum_thiserror;
  };
}
