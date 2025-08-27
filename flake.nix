{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs:
    let
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs
          [
            "x86_64-linux"
            "aarch64-linux"
          ]
          (
            system:
            f {
              pkgs = import inputs.nixpkgs {
                inherit system;
                overlays = [
                  inputs.rust-overlay.overlays.default
                  (final: prev: {
                    rustToolchain = prev.rust-bin.stable.latest.default.override {
                      extensions = [
                        "rust-analyzer"
                        "rust-src"
                      ];
                    };
                    rustfmt = prev.lib.hiPrio prev.rust-bin.nightly.latest.rustfmt;
                    deps = with prev; [
                      expat
                      fontconfig
                      freetype
                      freetype.dev
                      libGL
                      wayland
                      libxkbcommon
                      vulkan-loader
                    ];

                    RUSTFLAGS = "-C link-arg=-Wl,-rpath,${final.lib.makeLibraryPath final.deps}";
                  })

                ];
              };
            }
          );

      package =
        pkgs: name: description:
        pkgs.rustPlatform.buildRustPackage rec {
          inherit name;

          src = pkgs.lib.cleanSource ./.;

          env = {
            RUSTFLAGS = pkgs.RUSTFLAGS;
            ICED_BACKEND = "wgpu";
          };

          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          meta = {
            inherit description;
            homepage = "https://github.com/janTatesa/annoyodoro";
            license = pkgs.lib.licenses.mit;
            mainProgram = name;
          };
        };
    in
    {
      packages = forEachSupportedSystem (
        { pkgs }:
        rec {
          default = annoyodoro;
          annoyodoro = package pkgs "annoyodoro" "Annoying pomodoro timer";
          annoyodoro-break-timer = package pkgs "annoyodoro-break-timer" "Break timer for annoyodoro";
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              pkg-config
              rustfmt
              rustToolchain
              openssl
            ];

            env = {
              ICED_BACKEND = "wgpu";
              RUST_BACKTRACE = 1;
              RUSTFLAGS = pkgs.RUSTFLAGS;
            };
          };
        }
      );
    };
}
