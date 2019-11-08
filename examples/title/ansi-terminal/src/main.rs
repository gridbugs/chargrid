extern crate prototty_ansi_terminal;

// Assuming the title and its views were defined here
extern crate prototty_title;

use prototty_title::*;

fn main() {
    let mut context = prototty_ansi_terminal::Context::new().unwrap();

    let title = Title {
        width: 20,
        text: "My Title".to_string(),
    };

    // render the title using the DemoTitleView
    context
        .render(&mut DemoTitleView, &title, prototty_ansi_terminal::col_encode::FromTermInfoRgb)
        .unwrap();

    // exit after a key is pressed
    context.wait_input().unwrap();
}
