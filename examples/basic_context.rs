extern crate prototty;
extern crate cgmath;

use prototty::*;

fn main() {

    let div = AbsDiv::new((20, 10)).into_handle();
    let root_element = ElementHandle::from(div.clone());

    let text = Text::new("blah", (7, 3)).into_handle();

    div.insert("a", text.clone(), (0, 0), None);
    div.get("a").unwrap().text().unwrap().set("abcdefghijklnmopqrstuvwxyz");
    div.update_coord("a", (2, 2)).unwrap();

    div.insert("b", Text::new("HELLO\nWORLD", (8, 3)), (5, 4), Some(-1));

    let mut ctx = Context::new().unwrap();
    ctx.render(&root_element).unwrap();

    ctx.wait_input().unwrap();
}
