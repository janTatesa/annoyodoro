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

      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.rust-overlay.overlays.default
                inputs.self.overlays.default
              ];
            };
          }
        );
    in
    {
      overlays.default = final: prev: {
        rustToolchain = prev.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rustfmt"
          ];
        };

        deps = [
          prev.wayland
          prev.libxkbcommon
          prev.vulkan-loader
        ];
      };

      packages = forEachSupportedSystem (
        { pkgs }:
        let
          homepage = "https://github.com/TadoTheMiner/annoyodoro";
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
              description = "Gui for annoyodoro";
              inherit homepage license;
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
              rustToolchain
              openssl
              pkg-config
              cargo-deny
              cargo-edit
              cargo-watch
              rust-analyzer
            ];

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
              RUSTFLAGS = "-C link-args=-Wl,-rpath,${pkgs.lib.makeLibraryPath pkgs.deps}";
              ANNOYODORO_LOG = "debug";
            };
          };
        }
      );
    };
}
