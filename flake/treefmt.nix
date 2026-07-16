{
  inputs,
  lib,
  self,
  ...
}: {
  imports = [inputs.treefmt-nix.flakeModule];

  perSystem = {config, ...}: {
    treefmt.projectRoot = lib.cleanSourceWith {
      src = self.outPath;
      filter = path: type:
        !(type == "directory" && baseNameOf path == ".git")
        && !(lib.hasInfix "/.git/" (toString path));
    };

    treefmt.programs.alejandra.enable = true;
    treefmt.programs.rustfmt.enable = true;
    treefmt.settings.global.excludes = [
      ".git/**"
      "target/**"
    ];

    formatter = config.treefmt.build.wrapper;
  };
}
