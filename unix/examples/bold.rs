extern crate prototty;
extern crate prototty_unix;

use prototty::*;
use prototty_unix::*;

struct BoldTestView;

impl View<()> for BoldTestView {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        _data: &(),
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        StringView.view("Hello, World!", context.add_offset(Coord::new(1, 1)), grid);
        RichStringView::with_info(TextInfo::default().bold()).view(
            "Hello, World!",
            context.add_offset(Coord::new(1, 3)),
            grid,
        );
    }
}

fn main() {
    let mut context = Context::new().unwrap();
    context.render(&mut BoldTestView, &()).unwrap();
    loop {
        match context.wait_input().unwrap() {
            prototty_inputs::ETX => break,
            _ => (),
        }
    }
}
