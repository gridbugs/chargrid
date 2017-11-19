extern crate prototty;
extern crate cgmath;

use cgmath::Vector2;
use prototty::*;

fn main() {
//    let mut ctx = Context::new().unwrap();

    let mut div = AbsDiv::new((20, 10)).into_handle();

    let text = Text::new("Hello, World!", (15, 2)).into_handle();

    div.insert("hello", text.clone(), Vector2::new(1, 10), 1);

    div.get("hello").unwrap().text().unwrap().set("Boop");
    div.update_coord("hello", (2, 10)).unwrap();

    println!("{:?}", div);
/*
    ctx.render(div);
    */
}
