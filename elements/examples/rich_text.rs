extern crate prototty;
extern crate prototty_elements;
extern crate cgmath;
extern crate ansi_colour;

use ansi_colour::colours;
use prototty::*;
use prototty_elements::*;

fn main() {

    let text = RichText::new(vec![
        ("This text is red. ", TextInfo::default().foreground_colour(colours::RED)),
        ("This text is underlined. ", TextInfo::default().underline()),
        ("This text is bold. ", TextInfo::default().bold()),
        ("This text is blue and underlined. ", TextInfo::default().foreground_colour(colours::BLUE).underline()),
    ], (14, 6));

    let mut ctx = Context::new().unwrap();
    ctx.render(&text).unwrap();
    ctx.wait_input().unwrap();
}
