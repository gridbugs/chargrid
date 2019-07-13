extern crate prototty;
extern crate prototty_unix;

use prototty::*;
use prototty_unix::*;

struct BoldTestView;

impl View<()> for BoldTestView {
    fn view<F: Frame, R: ViewTransformRgb24>(&mut self, (): (), context: ViewContext<R>, frame: &mut F) {
        RichStringViewSingleLine.view(
            ("Hello, World!", Style::new().with_bold(true)),
            context.add_offset(Coord::new(1, 1)),
            frame,
        );
    }
}

fn main() {
    let mut context = Context::new().unwrap();
    context.render(&mut BoldTestView, ()).unwrap();
    loop {
        match context.wait_input().unwrap() {
            prototty_inputs::ETX => break,
            _ => (),
        }
    }
}
