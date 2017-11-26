extern crate prototty;
extern crate cgmath;

use prototty::*;

fn main() {

    let text = Text::new("abcdefghijklnmopqrstuvwxyz", (7, 3));
    let border = BorderContainer::new(text.clone());
    let root_element = ElementHandle::from(border.clone());

    let mut ctx = Context::new().unwrap();
    ctx.render(&root_element).unwrap();

    ctx.wait_input().unwrap();
}
