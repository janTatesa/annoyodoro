{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
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
                      wayland
                      libxkbcommon
                      vulkan-loader
                    ];
                  })
                ];
              };
            }
          );
    in
    {
      packages = forEachSupportedSystem (
        { pkgs }:
        {
          default =
            (pkgs.makeRustPlatform {
              rustc = pkgs.rustToolchain;
              cargo = pkgs.rustToolchain;
            }).buildRustPackage
              rec {
                name = "annoyodoro";
                src = pkgs.lib.cleanSource ./.;
                env = {
                  ICED_BACKEND = "wgpu";
                  LUCIDE_PATH = "${pkgs.lucide}/share/fonts/truetype/Lucide.ttf";
                };

                buildInputs = pkgs.deps;
                nativeBuildInputs = [ pkgs.pkg-config ];
                cargoLock = {
                  lockFile = ./Cargo.lock;
                  allowBuiltinFetchGit = true;
                };

                postFixup = ''
                  rpath=$(patchelf --print-rpath $out/bin/${name})
                  patchelf --set-rpath "$rpath:${pkgs.lib.makeLibraryPath pkgs.deps}" $out/bin/${name}
                '';

                meta = {
                  description = "An annoying pomodoro timer";
                  homepage = "https://github.com/janTatesa/annoyodoro";
                  license = pkgs.lib.licenses.mit;
                  mainProgram = name;
                };
              };
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
              dbus
            ];

            env = {
              ICED_BACKEND = "wgpu";
              RUST_BACKTRACE = 1;
              RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath pkgs.deps}";
              LUCIDE_PATH = "${pkgs.lucide}/share/fonts/truetype/Lucide.ttf";
            };
          };
        }
      );
    };
}
