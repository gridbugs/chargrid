extern crate prototty;
extern crate prototty_unix;

use prototty::render::*;
use prototty::text::*;
use prototty_unix::*;

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
            prototty::input::inputs::ETX => break,
            _ => (),
        }
    }
}
