extern crate prototty;
extern crate prototty_unix;

use prototty::*;
use prototty_unix::*;

struct BoldTestView;

impl View<()> for BoldTestView {
    fn view<G: ViewGrid>(&mut self, _data: &(), offset: Coord, depth: i32, grid: &mut G) {
        StringView.view("Hello, World!", offset + Coord::new(1, 1), depth, grid);
        RichStringView::with_info(TextInfo::default().bold()).view(
            "Hello, World!",
            offset + Coord::new(1, 3),
            depth,
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
