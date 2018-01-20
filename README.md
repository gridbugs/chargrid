# Prototty

This repo contains a collection of crates relating to rendering grids of
characters. Cells in the grid have characters, foreground and background
colours, and attributes bold and underline.

- [prototty](https://github.com/stevebob/prototty/tree/master/prototty) defines
  some types and traits which allow you to describe how a type should be
  rendered to a grid of characters. It also defines an input type.
- [prototty-common](https://github.com/stevebob/prototty/tree/master/common) is
  a collection of common user interface element types and corresponding views so
  they may be rendered with prototty.
- [prototty-unix](https://github.com/stevebob/prototty/tree/master/unix) is a
  prototty frontend for unix (ansi) terminals. It can render views to a unix
  terminal, and get input from the terminal, normalized to prototty's input
  type.
- [prototty-glutin](https://github.com/stevebob/prototty/tree/master/unix) is a
  prototty frontend for opengl. It can create a window, render views to that
  window, and get input from the window, normalized to prototty's input type.
- [prototty-wasm](https://github.com/stevebob/prototty/tree/master/wasm) is a
  prototty frontend for web assembly. It renders views into memory in a format
  which can be easily unpacked in javascript, and contains functions for
  normalizing javascript key codes and key mods into prototty's input type. It's
  intended to be used with [prototty-terminal-js](https://github.com/stevebob/prototty-terminal-js) - a
  javascript library for unpacking the output of prototty-wasm and drawing it to
  the screen, and periodically sending input to the wasm program.
- [prototty-grid](https://github.com/stevebob/prototty/tree/master/grid) defines a
  data structure for storing a grid of cells, and a common cell representation.
  It's used by the frontend crates in this repo, and would simplify the
  implementation of any further frontends.

## Example

See [prototty-tetris](https://github.com/stevebob/prototty-tetris) for an
example application built with these libraries.
