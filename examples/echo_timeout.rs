extern crate prototty;
use std::io::Write;
use std::time::Duration;
use prototty::{Terminal, Input};

const ESCAPE: char = '\u{1b}';
const ETX: char = '\u{3}';

fn main() {
    let error = {
        let mut terminal = Terminal::new().unwrap();

        loop {
            let input = match terminal.wait_input_timeout(Duration::from_millis(1000)) {
                Ok(Some(input)) => input,
                Ok(None) => {
                    writeln!(&mut terminal, "\rtimeout").unwrap();
                    terminal.flush().unwrap();
                    continue;
                }
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
