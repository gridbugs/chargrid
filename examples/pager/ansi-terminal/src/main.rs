extern crate pager_prototty;
extern crate prototty_ansi_terminal;

use pager_prototty::*;
use prototty_ansi_terminal::*;
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
        context.render(&mut view, &state, col_encode::FromTermInfoRgb).unwrap();
        let input = context.wait_input().unwrap();
        match state.tick((&[input]).iter().cloned(), &view) {
            None => (),
            Some(ControlFlow::Exit) => break,
        }
    }
    Ok(())
}
