extern crate prototty;
extern crate prototty_unix;

use prototty::*;
use prototty_unix::*;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut string = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut string)?;
    let mut context = Context::new().unwrap();
    let mut view = RichTextView::default();
    loop {
        context.render(&mut view, &["Hello, World!"]).unwrap();
        match context.wait_input().unwrap() {
            prototty_inputs::ETX | prototty_inputs::ESCAPE => break,
            _ => (),
        }
    }
    Ok(())
}
