{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  packages = with pkgs; [
    rustc
    cargo
    rustPlatform.rustLibSrc
    rust-analyzer
    cargo-watch
    rustfmt
  ] ++ (import ./common.nix).projectDeps;

  # Allows rust-analyzer to find the rust source
  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

  # Without this graphical frontends can't find the GPU adapters
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath packages}";
}
