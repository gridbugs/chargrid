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
    let text_info = TextInfo::default()
        .foreground_colour(Rgb24::new(0, 255, 0))
        .background_colour(Rgb24::new(100, 100, 100))
        .bold();
    let scrollbar = PagerScrollbar {
        text_info: TextInfo::default().foreground_colour(Rgb24::new(127, 0, 0)),
        ..PagerScrollbar::default()
    };
    let mut pager = Pager::new(
        &string,
        context.size().unwrap() - Size::new(scrollbar.padding() + 1, 0),
        text_info,
    );
    loop {
        context
            .render(
                &mut PagerViewWithScrollbar(PagerView),
                &(&pager, &scrollbar),
            )
            .unwrap();
        match context.wait_input().unwrap() {
            prototty_inputs::ETX => break,
            ProtottyInput::Up => pager.up(),
            ProtottyInput::Down => pager.down(),
            _ => (),
        }
    }
    Ok(())
}
