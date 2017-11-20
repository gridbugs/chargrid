extern crate prototty;
extern crate cgmath;
extern crate terminal_colour;

use std::io::Write;
use std::time::Duration;
use std::thread;
use cgmath::Vector2;
use terminal_colour::colours;
use prototty::Terminal;

fn main() {
    let mut terminal = Terminal::new().unwrap();

    let hello_world = "\u{2764}Hello, World!\u{2764}";

    let size = terminal.size().unwrap();
    let mid = size / 2;
    let start = Vector2 {
        x: mid.x.saturating_sub(hello_world.len() as u16 / 2),
        .. mid
    };

    terminal.set_foreground_colour(colours::GREEN);
    terminal.set_background_colour(colours::RED);
    terminal.set_cursor(start).unwrap();
    terminal.set_bold();
    write!(&mut terminal, "\u{2764}Hello").unwrap();
    terminal.reset();
    write!(&mut terminal, ", ").unwrap();
    terminal.set_foreground_colour(colours::MAGENTA);
    terminal.set_background_colour(colours::BRIGHT_YELLOW);
    write!(&mut terminal, "World").unwrap();
    terminal.reset();
    terminal.set_underline();
    write!(&mut terminal, "!\u{2764}").unwrap();
    terminal.flush().unwrap();

    thread::sleep(Duration::from_millis(1000));
}
