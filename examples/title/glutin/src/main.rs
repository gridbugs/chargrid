extern crate prototty_glutin;

// Assuming the title and its views were defined here
extern crate prototty_title;

use prototty_glutin::prototty_input::{keys, Input};
use prototty_glutin::Size;
use prototty_title::*;

fn main() {
    // Use a builder to configure how text should be rendered.
    // This assumes "Hack-Regular.ttf" and "Hack-Bold.ttf" are in your "src" directory.
    let size = Size::new(320, 240);
    let mut context = prototty_glutin::ContextBuilder::new_with_font(include_bytes!("Hack-Regular.ttf"))
        .with_bold_font(include_bytes!("Hack-Bold.ttf"))
        .with_window_dimensions(size)
        .with_min_window_dimensions(size)
        .with_max_window_dimensions(size)
        .with_font_scale(32.0, 32.0)
        .with_cell_dimensions(Size::new(16, 32))
        .with_underline_position(28)
        .with_underline_width(2)
        .with_max_grid_size(Size::new(30, 30))
        .build()
        .unwrap();

    let title = Title {
        width: 20,
        text: "My Title".to_string(),
    };

    let mut running = true;
    while running {
        // render the title using the DemoTitleView
        context.render(&mut DemoTitleView, &title).unwrap();

        // exit after the window is closed
        context.poll_input(|input| {
            if input == Input::Keyboard(keys::ETX) {
                running = false;
            }
        });
    }
}
