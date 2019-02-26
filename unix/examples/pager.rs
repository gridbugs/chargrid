extern crate prototty;
extern crate prototty_unix;

use prototty::*;
use prototty_unix::*;

fn main() {
    let mut context = Context::new().unwrap();
    loop {
        match context.wait_input().unwrap() {
            prototty_inputs::ETX => break,
            _ => (),
        }
    }
}
