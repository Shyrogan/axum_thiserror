{
  perSystem = {
    config,
    pkgs,
    ...
  }: let
    rustToolchain = pkgs.rust-bin.stable.latest.default.override {
      extensions = [
        "rust-src"
        "rust-analyzer"
        "clippy"
        "rustfmt"
      ];
    };
  in {
    devShells.default = pkgs.mkShell {
      inputsFrom = [config.pre-commit.devShell];

      packages = [rustToolchain] ++ config.skills.enabledPackages;

      shellHook = config.skills.shellHook;

      env.RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
    };
  };
}
