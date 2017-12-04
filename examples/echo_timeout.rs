extern crate prototty;
use std::time::Duration;
use prototty::{Context, Input};

const ESCAPE: char = '\u{1b}';
const ETX: char = '\u{3}';

fn main() {
    let error = {
        let mut context = Context::new().unwrap();

        loop {
            let input = match context.wait_input_timeout(Duration::from_millis(1000)) {
                Ok(Some(input)) => input,
                Ok(None) => {
                    println!("timeout\r");
                    continue;
                }
                Err(e) => break Some(e),
            };

            if input == Input::Char(ESCAPE) || input == Input::Char(ETX) {
                break None;
            } else {
                println!("{:?}\r", input);
            }
        }
    };

    println!("error: {:?}", error);
}
