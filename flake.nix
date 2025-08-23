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
        rustToolchain = prev.rust-bin.stable.latest.default;
        rustfmt = prev.lib.hiPrio prev.rust-bin.nightly.latest.rustfmt;
        deps = with prev; [
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

      };

      packages = forEachSupportedSystem (import ./packaging.nix);
      devShells = forEachSupportedSystem (
        { pkgs }:
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              pkg-config
              rustfmt
              rustToolchain
              openssl
              cargo-deny
              cargo-edit
              cargo-watch
              rust-analyzer
            ];

            env = {
              ICED_BACKEND = "wgpu";
              RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath pkgs.deps}";
            };
          };
        }
      );
    };
}
