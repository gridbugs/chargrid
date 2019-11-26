# prototty\_graphical

[![Version](https://img.shields.io/crates/v/prototty_graphical.svg)](https://crates.io/crates/prototty_graphical)
[![Documentation](https://docs.rs/prototty_graphical/badge.svg)](https://docs.rs/prototty_graphical)

A graphical frontend for prototty which renders with opengl.

## Example

Let's continue the title example started
[here](https://github.com/stevebob/prototty/tree/master/prototty#example):

```rust
extern crate prototty;
extern crate prototty_graphical;

// Assuming the title and its views were defined here
extern crate prototty_title;

use prototty::{Renderer, inputs};
use prototty_title::*;

fn main() {

    // Use a builder to configure how text should be rendered.
    // This assumes "Hack-Regular.ttf" is in your "src" directory.
    let mut context = prototty_graphical::ContextBuilder::new_with_font(
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

Running this will produce the following output in a new window:

![Example](https://github.com/stevebob/prototty/blob/master/graphical/example.png)
