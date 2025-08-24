{
  inputs = {
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
      };
    };

    parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      parts,
      nci,
      ...
    }:

    parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "x86_64-aarch64"
      ];

      imports = [
        nci.flakeModule
      ];

      perSystem =
        {
          system,
          pkgs,
          lib,
          config,
          ...
        }:
        let
          deps = with pkgs; [
            expat
            fontconfig
            freetype
            freetype.dev
            libGL
            pkg-config
            wayland
            libxkbcommon
            vulkan-loader
          ];

          rustToolchain =
            pkgs:
            pkgs.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-analyzer"
                "rust-src"
              ];
            };
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.rust-overlay.overlays.default
            ];
          };

          devShells.default = config.nci.outputs.annoyodoro.devShell.overrideAttrs (old: {
            packages = [
              (lib.hiPrio pkgs.rust-bin.nightly.latest.rustfmt)
            ]
            ++ (old.packages or [ ]);

            env = {
              RUST_SRC_PATH = "${rustToolchain pkgs}/lib/rustlib/src/rust/library";
            };
          });

          packages = rec {
            default = annoyodoro;
            annoyodoro = config.nci.outputs.annoyodoro.packages.release;
            annoyodoro-break-timer = config.nci.outputs."annoyodoro-break-timer".packages.release;
          };

          nci = {
            toolchains.mkShell = rustToolchain;

            projects.annoyodoro.path = ./.;
            crates = {
              annoyodoro.runtimeLibs = deps;
              "annoyodoro-break-timer".runtimeLibs = deps;
            };
          };
        };
    };
}
