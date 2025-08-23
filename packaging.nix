{ pkgs }:
let
  homepage = "https://github.com/janTatesa/annoyodoro";
  license = pkgs.lib.licenses.mit;
in
rec {
  default = annoyodoro;
  annoyodoro = pkgs.rustPlatform.buildRustPackage rec {
    name = "annoyodoro";

    src = pkgs.lib.cleanSource ./.;
    buildInputs = pkgs.deps;

    cargoLock = {
      lockFile = ./Cargo.lock;
      # Allow dependencies to be fetched from git and avoid having to set the outputHashes manually
      allowBuiltinFetchGit = true;
    };
    meta = {
      description = "An annoying pomodoro timer";
      inherit homepage license;
      mainProgram = name;
    };
  };

  annoyodoro-break-timer = pkgs.rustPlatform.buildRustPackage rec {
    name = "annoyodoro-break-timer";

    src = pkgs.lib.cleanSource ./.;

    buildInputs = pkgs.deps;
    cargoLock = {
      lockFile = ./Cargo.lock;
      # Allow dependencies to be fetched from git and avoid having to set the outputHashes manually
      allowBuiltinFetchGit = true;
    };

    meta = {
      description = "The break timer for annoyodoro";
      inherit homepage license;
      mainProgram = name;
    };
  };
}
