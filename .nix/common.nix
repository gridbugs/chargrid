# Nix shell with rust compiler and dependencies for libraries and examples
#
# Known issues:
# - in a pure shell, alsa-lib can't connect to pulseaudio
# - in non-NixOS environments, there are errors when initializing audio of the form:
#   ALSA lib dlmisc.c:337:(snd_dlobj_cache_get0) Cannot open shared library libasound_module_pcm_pulse.so
# - in non-NixOS environments, there is an error when starting a nix-shell:
#   * Error: Unable to use "ps" to scan for ssh-agent processes
#   * Error: Please report to x48rph@gmail.com via http://bugs.gentoo.org
#   ^^^ This error goes away if the `LD_LIBRARY_PATH = ...` line is removed (but that line is necessary for graphical frontends to work)

let
  moz_overlay_url = "https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz";
  moz_overlay = import (builtins.fetchTarball moz_overlay_url);
  nixpkgs = import <nixpkgs> {
    overlays = [ moz_overlay ];
  };
  rust_override = {
    extensions = [ "rust-src" "rust-analysis" ];
    targets = [ "wasm32-unknown-unknown" ];
  };
in
  with nixpkgs;
  {
    stable = nixpkgs.latest.rustChannels.stable.rust.override rust_override;
    nightly = nixpkgs.latest.rustChannels.nightly.rust.override rust_override;
    rustChannelOf = desc: (rustChannelOf desc).rust.override rust_override;
    mkDerivation = { rust }:
      stdenv.mkDerivation rec {
        name = "moz_overlay_shell";
        buildInputs = [
          rust
          rust-analyzer
          cargo-watch

          # project-specific dependencies
          lld
          clang
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
          libGL
          bzip2
          nodejs-16_x
          wasm-pack
        ];

        # Enable backtraces on panics
        RUST_BACKTRACE = 1;

        # Without this graphical frontends can't find the GPU adapters
        LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
      };
  }
