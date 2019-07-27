extern crate pager_prototty;
extern crate prototty_unix;

use pager_prototty::*;
use prototty_unix::*;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut string = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    handle.read_to_string(&mut string)?;
    let mut context = Context::new().unwrap();
    let mut view = AppView::new();
    let mut state = AppState::new(string);
    loop {
        context.render(&mut view, &state, encode_colour::FromTermInfoRgb).unwrap();
        let input = context.wait_input().unwrap();
        match state.tick((&[input]).iter().cloned(), &view) {
            None => (),
            Some(ControlFlow::Exit) => break,
        }
    }
    Ok(())
}
