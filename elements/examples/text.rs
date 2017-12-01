extern crate prototty;
extern crate prototty_elements;

use prototty::*;
use prototty_elements::*;

fn main() {
    let mut context = Context::new().unwrap();
    let text = Text::new("abcdefghijklmnopqrstuvwxyz", (5, 5));
    context.render(&text).unwrap();
    context.wait_input().unwrap();
}
