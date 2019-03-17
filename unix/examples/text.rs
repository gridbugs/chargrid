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
    let mut view = Aligned::new(
        Bordered::new(
            Bounded::new(
                VerticalScrolled::new(
                    RichTextView::new(wrap::Word::new()),
                    VerticalScrollbar::default(),
                ),
                Size::new(40, 40),
            ),
            Border::default(),
        ),
        Align::new(AlignX::Centre, AlignY::Centre),
    );
    loop {
        context
            .render(
                &mut view,
                &[
                    ("Hello, World!\nblah\nblah blah ", Style::default()),
                    (
                        "blue\n",
                        Style {
                            foreground: colours::BRIGHT_BLUE,
                            bold: true,
                            ..Default::default()
                        },
                    ),
                    ("User string:\n", Default::default()),
                    (
                        string.as_ref(),
                        Style {
                            background: colours::RED,
                            underline: true,
                            ..Default::default()
                        },
                    ),
                ],
            )
            .unwrap();
        let scroll: &mut VerticalScrolled<_> = &mut view.view.view.view;
        match context.wait_input().unwrap() {
            prototty_inputs::ETX | prototty_inputs::ESCAPE | ProtottyInput::Char('q') => {
                break;
            }
            ProtottyInput::Up => scroll.scroll_up_line(),
            ProtottyInput::Down => scroll.scroll_down_line(),
            ProtottyInput::PageUp => scroll.scroll_up_page(),
            ProtottyInput::PageDown => scroll.scroll_down_page(),
            ProtottyInput::Home | ProtottyInput::Char('g') => scroll.scroll_to_top(),
            ProtottyInput::End | ProtottyInput::Char('G') => scroll.scroll_to_bottom(),
            _ => (),
        }
    }
    Ok(())
}
