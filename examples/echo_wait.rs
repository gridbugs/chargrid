extern crate prototty;
use std::io::Write;
use prototty::{Terminal, Input};

const ESCAPE: char = '\u{1b}';
const ETX: char = '\u{3}';

fn main() {
    let error = {
        let mut terminal = Terminal::new().unwrap();

        loop {
            let input = match terminal.wait_input() {
                Ok(input) => input,
                Err(e) => break Some(e),
            };

            if input == Input::Char(ESCAPE) || input == Input::Char(ETX) {
                break None;
            } else {
                writeln!(&mut terminal, "\r{:?}", input).unwrap();
                terminal.flush().unwrap();
            }
        }
    };

    println!("error: {:?}", error);
}
