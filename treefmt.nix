{ pkgs, ... }:
{
  projectRootFile = "flake.nix";
  programs = {
    nixfmt.enable = true;
    rustfmt.enable = true;
    actionlint.enable = true;
  };
}