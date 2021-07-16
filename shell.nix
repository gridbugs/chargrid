let
  moz_overlay = import (builtins.fetchTarball https://github.com/andersk/nixpkgs-mozilla/archive/stdenv.lib.tar.gz);
  nixpkgs = import <nixpkgs> {
    overlays = [ moz_overlay ];
  };
  ruststable = (nixpkgs.latest.rustChannels.stable.rust.override {
    extensions = [ "rust-src" "rust-analysis" ];}
  );
in
  with nixpkgs;
  stdenv.mkDerivation rec {
    name = "rust";
    buildInputs = [
      rustup
      ruststable
      pkg-config
      alsaLib
      udev
      xorg.libX11
      xorg.libXcursor
      xorg.libXrandr
      xorg.libXi
      vulkan-loader
      vulkan-tools
    ];

    LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
  }
