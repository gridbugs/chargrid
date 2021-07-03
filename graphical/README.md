# chargrid\_graphical

[![Version](https://img.shields.io/crates/v/chargrid_graphical.svg)](https://crates.io/crates/chargrid_graphical)
[![Documentation](https://docs.rs/chargrid_graphical/badge.svg)](https://docs.rs/chargrid_graphical)

A graphical frontend for chargrid which renders with wgpu.

## Dependencies

### Linux

On linux this renders with vulkan. You will need a vulkan loader and vulkan
drivers installed in order to run graphical chargrid applications.  This often
takes the form of a library named "libvulkan.so".  If you encounter the
following error when running a graphical chargrid application it means you're
missing a vulkan dependency:
```
Failed to initialize graphical context: FailedToRequestGraphicsAdapter
```

#### NixOS

The following shell.nix creates an environment in which graphical chargrid applications can
be built and run:
```
with import <nixpkgs> {};
pkgs.mkShell {
  buildInputs = [
    gtk3 glib
    pkgconfig
    xorg.libX11
    vulkan-loader
  ];
  shellHook = ''
    export LD_LIBRARY_PATH="${vulkan-loader}/lib"
  '';
}
```

## Compiling Shaders

To simplify building/runnig, pre-compiled shaders are checked into the repo. After changing the
shader source, run the `compile-shaders.sh` script to update the compiled shaders. This script
depends on the [shader-translator](https://crates.io/crates/shader-translator) tool.

```
cargo install shader-translator
./compile-shaders.sh
```
