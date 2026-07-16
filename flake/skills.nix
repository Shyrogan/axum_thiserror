{inputs, ...}: {
  imports = [inputs.skills-parts.flakeModules.default];

  # This repository's own skills are discovered from `skills/<name>/SKILL.md`.
  # Also install the skills exported by skills-parts (flake-parts and
  # skills-parts usage guides) into the dev shell.
  perSystem.skills = [inputs.skills-parts.skills];
}
