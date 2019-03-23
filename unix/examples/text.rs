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
            Border {
                title_style: Style {
                    bold: Some(true),
                    foreground: Some(rgb24(0, 255, 0)),
                    background: Some(rgb24(0, 64, 0)),
                    ..Default::default()
                },
                padding: BorderPadding {
                    right: 0,
                    left: 2,
                    top: 1,
                    bottom: 1,
                },
                ..Border::default_with_title("Pager")
            },
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
                            foreground: Some(colours::BRIGHT_BLUE),
                            bold: Some(true),
                            ..Default::default()
                        },
                    ),
                    ("User string:\n", Default::default()),
                    (
                        string.as_ref(),
                        Style {
                            background: Some(colours::RED),
                            underline: Some(true),
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
            ProtottyInput::MouseScroll { direction, .. } => match direction {
                ScrollDirection::Up => scroll.scroll_up_line(),
                ScrollDirection::Down => scroll.scroll_down_line(),
                _ => (),
            },
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
