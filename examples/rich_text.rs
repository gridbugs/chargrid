extern crate prototty;
extern crate cgmath;
extern crate ansi_colour;

use ansi_colour::colours;
use prototty::*;

fn main() {

    let text = RichText::new(&[
        ("This text is red. ".to_string(), TextInfo::default().foreground_colour(colours::RED)),
        ("This text is underlined. ".to_string(), TextInfo::default().underline()),
        ("This text is bold. ".to_string(), TextInfo::default().bold()),
        ("This text is blue and underlined. ".to_string(), TextInfo::default().foreground_colour(colours::BLUE).underline()),
    ], (14, 6));
    let root_element = ElementHandle::from(text.clone());

    let mut ctx = Context::new().unwrap();
    ctx.render(&root_element).unwrap();

    ctx.wait_input().unwrap();
}
