let
  moz_overlay = import (builtins.fetchTarball https://github.com/stevebob/nixpkgs-mozilla/archive/with-stdenv.lib-fix.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  ruststable = (nixpkgs.latest.rustChannels.stable.rust.override {
    extensions = [ "rust-src" "rust-analysis" ];
  });
in
with nixpkgs;
stdenv.mkDerivation rec {
  name = "moz_overlay_shell";
  buildInputs = [
    ruststable

    # project-specific dependencies
    pkg-config
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
  ];

  RUST_BACKTRACE = 1;
}
