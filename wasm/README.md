# prototty\_wasm

[![Version](https://img.shields.io/crates/v/prototty_wasm.svg)](https://crates.io/crates/prototty_wasm)
[![Documentation](https://docs.rs/prototty_wasm/badge.svg)](https://docs.rs/prototty_wasm)

A prototty frontend for web assembly. It renders views into memory in a format
which can be easily unpacked in javascript, and contains functions for
normalizing javascript key codes and key mods into prototty's input type. It's
intended to be used with [prototty-terminal-js](https://github.com/stevebob/prototty/terminal-js) - a
javascript library for unpacking the output of prototty-wasm and drawing it to
the screen, and periodically sending input to the wasm program.

## Example

Let's continue the title example started
[here](https://github.com/stevebob/prototty/tree/master/prototty#example).
We'll be creating a webpage which will display the output of rendering a title with
a `DemoTitleView`, written mostly in rust, compiled to web assembly.

Unlike the unix and glutin frontends, you don't make a binary crate when using
the wasm frontend. Instead, the entire application lives in a library, exposing
an interface, which will be consumed by some javascript. The library is compiled
to web assembly, and loaded into page, where it interacts with javascript. All
the javascript interaction is handled by a library called
[prototty-terminal-js](https://github.com/stevebob/prototty/terminal-js).

### The rust part
For our title example, the library looks like:

```rust
extern crate prototty;
extern crate prototty_wasm;

// Assuming the title and its views were defined here
extern crate prototty_title;

use prototty::Renderer;
use prototty_title::*;

// Define a type containing the entire application state.
pub struct App {
    title: Title,
    context: prototty_wasm::Context,
}

// Implement a function "alloc_app", which allocates the
// application state, returning a pointer to it. This will
// be called by the prototty-terminal-js library.
//
// This function takes a rng seed, which we ignore here.
#[no_mangle]
pub extern "C" fn alloc_app(_seed: usize) -> *mut App {
    let context = prototty_wasm::Context::new();
    let title = Title {
        width: 20,
        text: "My Title".to_string(),
    };
    let app = App { title, context };

    prototty_wasm::alloc::into_boxed_raw(app)
}

// Implement a function "tick", which is called periodically
// by prototty-terminal-js. It's passed a pointer to the app
// state (allocated by "alloc_app"), and some information about
// inputs and the time that passed since it was last called,
// which we ignore here.
#[no_mangle]
pub unsafe fn tick(app: *mut App,
                   _key_codes: *const u8,
                   _key_mods: *const u8,
                   _num_inputs: usize,
                   _period_millis: f64) {

    (*app).context.render(&DemoTitleView, &(*app).title).unwrap();
}
```

Now let's get this compiling to web assembly. You'll need to install the web
assembly rust target:

```
$ rustup target add wasm32-unknown-unknown
```

To prevent the wasm binary from being really big, install `wasm-gc`:

```
$ cargo install --git https://github.com/alexcrichton/wasm-gc
```

Add the following to the crate's `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]
```

And build your web assembly binary:

```
$ cargo build --release
```

Make a directory to house the webpage's files:

```
$ mkdir dist
```

Use `wasm-gc` to make the final binary. Here, "title\_wasm" is the name of the
crate.

```
$ wasm-gc target/wasm32-unknown-unknown/release/title_wasm.wasm dist/title_wasm.wasm
```

### The javascript part

Our goal is to make a page with our wasm binary built in the previous stage,
and the prototty-terminal-js library. We'll use
[webpack](https://webpack.js.org/) to bundle the library for client-side use.
First install webpack and prototty-terminal-js.

```
npm install --save-dev webpack
npm install --save prototty-terminal-js
```

This will create a `node_modules` directory containing webpack,
prototty-terminal-js, and their dependencies.

To actually start the application, we'll need to write a tiny bit of javascript.
Put the following in `index.js`:

```javascript
import prototty from 'prototty-terminal-js';

let protottyTerminal = document.getElementById("protottyTerminal");

prototty.loadProtottyApp("title_wasm.wasm", 20, 20, protottyTerminal, env).then(app => {
    app.start();
});
```

We'll use webpack to take this code, and the prototty-terminal-js, and any
dependencies it has, and bundle them into a single javascript file.

Put the following webpack configuration in a file called webpack.config.js:
```javascript
module.exports = {
  entry: './index.js',
  devtool: 'source-map',
  output: {
    filename: 'dist/bundle.js'
  },
};
```

Now run webpack:

```
$ node_modules/webpack/bin/webpack.js
```

This will create a file `dist/bundle.js`.

Now make a simple html file:

```html
<!doctype html>
<html>
  <head>
<style>
body {
  overflow: hidden;
  background-color: black;
}
</style>
  </head>
  <body>
    <div id="protottyTerminal"></div>
    <script src="bundle.js"></script>
  </body>
</html>
```

Last step: serve the page!

```
$ cd dist
$ python3 -m http.server 8000
```

Head over to your browser. You should see the following:

![Example](https://github.com/stevebob/prototty/blob/master/wasm/example.png)
