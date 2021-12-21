{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell rec {
  packages = with pkgs; [
    rustc
    cargo
    rust-analyzer
    cargo-watch
  ] ++ (import ./common.nix).projectDeps;

  # Without this graphical frontends can't find the GPU adapters
  LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath packages}";
}
