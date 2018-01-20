# prototty\_glutin

[![Version](https://img.shields.io/crates/v/prototty_glutin.svg)](https://crates.io/crates/prototty_glutin)
[![Documentation](https://docs.rs/prototty_glutin/badge.svg)](https://docs.rs/prototty_glutin)

A prototty frontend for opengl which uses [glutin](https://github.com/tomaka/glutin)
for window creation and input. Provides a `Context` which can render a view to
a window, and access inputs.


## Example

Let's continue the title example started
[here](https://github.com/stevebob/prototty/tree/master/prototty#example):

```rust
extern crate prototty;
extern crate prototty_glutin;

// Assuming the title and its views were defined here
extern crate prototty_title;

use prototty::{Renderer, inputs};
use prototty_title::*;

fn main() {

    // Use a builder to configure how text should be rendered
    let mut context = prototty_glutin::ContextBuilder::new_with_font(
        include_bytes!("Hack-Regular.ttf"))
        .with_window_dimensions(320, 240)
        .with_font_scale(32.0, 32.0)
        .with_cell_dimensions(16, 32)
        .with_underline_position(28)
        .with_underline_width(2)
        .with_max_grid_size(30, 30)
        .build().unwrap();

    let title = Title {
        width: 20,
        text: "My Title".to_string(),
    };

    let mut running = true;
    while running {

        // render the title using the DemoTitleView
        context.render(&DemoTitleView, &title).unwrap();

        // exit after the window is closed
        context.poll_input(|input| {
            if input == inputs::ETX {
                running = false;
            }
        });
    }
}
```

Running this will produce the following output:

![Example](https://github.com/stevebob/prototty/blob/master/glutin/example.png)
