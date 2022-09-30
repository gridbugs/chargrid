{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  packages = with pkgs; [
    cmake
    rustc
    cargo
    rustPlatform.rustLibSrc
    rust-analyzer
    cargo-watch
    rustfmt
    pkg-config
    udev
    alsaLib
    libao
    openal
    libpulseaudio
    fontconfig
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    vulkan-loader
    vulkan-tools
    libGL
    bzip2
    nodejs-16_x
    wasm-pack
  ];

  # Allows rust-analyzer to find the rust source
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

  # Without this graphical frontends can't find the GPU adapters
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath packages}";
}
