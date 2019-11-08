extern crate prototty;
extern crate prototty_ansi_terminal;

use prototty::input::*;
use prototty::render::*;
use prototty::text::*;
use prototty_ansi_terminal::*;

struct BoldTestView;

impl View<()> for BoldTestView {
    fn view<F: Frame, C: ColModify>(&mut self, (): (), context: ViewContext<C>, frame: &mut F) {
        RichStringViewSingleLine.view(
            ("Hello, World!", Style::new().with_bold(true)),
            context.add_offset(Coord::new(1, 1)),
            frame,
        );
    }
}

fn main() {
    let mut context = Context::new().unwrap();
    context
        .render(&mut BoldTestView, (), col_encode::FromTermInfoRgb)
        .unwrap();
    loop {
        match context.wait_input().unwrap() {
            Input::Keyboard(keys::ETX) => break,
            _ => (),
        }
    }
}
