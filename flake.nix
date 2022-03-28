{
  description = "Rain Forest";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, flake-compat, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      with pkgs;
      {
        devShell = mkShell rec {
          buildInputs = [
            # General C Compiler/Linker/Tools
            lld
            clang
            pkg-config
            openssl

            # Rust Compiler
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analysis" ];
              targets = [ "wasm32-unknown-unknown" ];
            })
            rust-analyzer
            cargo-watch

            # Graphics and Audio Dependencies
            alsaLib
            libao
            openal
            libpulseaudio
            udev
            xorg.libX11
            xorg.libXcursor
            xorg.libXrandr
            xorg.libXi
            vulkan-loader
            vulkan-tools
            libGL
            bzip2

            # JS/Wasm Deps
            nodejs-17_x
            wasm-pack
          ];

          # Allows rust-analyzer to find the rust source
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

          # Without this graphical frontends can't find the GPU adapters
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";

        };
      }
    );
}
