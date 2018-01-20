# prototty\_unix

[![Version](https://img.shields.io/crates/v/prototty_unix.svg)](https://crates.io/crates/prototty_unix)
[![Documentation](https://docs.rs/prototty_unix/badge.svg)](https://docs.rs/prototty_unix)

A prototty frontend for unix terminals. Provides a `Context` which can render a
view to the terminal, and several ways to get input from the terminal.

## Example

Let's continue the title example started
[here](https://github.com/stevebob/prototty/tree/master/prototty#example):

```rust
extern crate prototty;
extern crate prototty_unix;

// Assuming the title and its views were defined here
extern crate prototty_title;

use prototty::Renderer;
use prototty_title::*;

fn main() {

    let mut context = prototty_unix::Context::new().unwrap();

    let title = Title {
        width: 20,
        text: "My Title".to_string(),
    };

    // render the title using the DemoTitleView
    context.render(&DemoTitleView, &title).unwrap();

    // exit after a key is pressed
    context.wait_input().unwrap();
}
```

Running this will produce the following output:
![Example](https://github.com/stevebob/prototty/tree/master/prototty_unix/example.png)
