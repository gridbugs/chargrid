# Chargrid

[![test](https://github.com/gridbugs/chargrid/actions/workflows/test.yml/badge.svg)](https://github.com/gridbugs/chargrid/actions/workflows/test.yml)
[![dependency status](https://deps.rs/repo/github/gridbugs/chargrid/status.svg)](https://deps.rs/repo/github/gridbugs/chargrid)

This repo contains a collection of crates relating to rendering grids of
characters. Cells in the grid have characters, foreground and background
colours, and attributes bold and underline.

## Nix

To set up a shell with an installation of rust and external dependencies:
```
nix-shell
```

For nightly rust:
```
nix-shell nix/nightly.nix
```

## Debug Environment

Source the script `debug_env_linux.sh` to set cargo environment variables for faster builds:
```
. debug_env_linux.sh
```
