extern crate prototty;
extern crate cgmath;

use std::io::Write;
use std::time::Duration;
use std::thread;
use cgmath::Vector2;
use prototty::Terminal;

fn main() {
    let mut terminal = Terminal::new().unwrap();

    let hello_world = "Hello, World!";

    let size = terminal.size().unwrap();
    let mid = size / 2;
    let start = Vector2 {
        x: mid.x.saturating_sub(hello_world.len() as u16 / 2),
        .. mid
    };

    terminal.move_cursor(start).unwrap();
    writeln!(&mut terminal, "{}", hello_world).unwrap();
    terminal.flush().unwrap();

    thread::sleep(Duration::from_millis(1000));
}
