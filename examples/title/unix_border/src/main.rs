extern crate prototty;
extern crate prototty_unix;
extern crate prototty_common;

// Assuming the title and its views were defined here
extern crate prototty_title;

use prototty::{Renderer, View, ViewSize, ViewGrid, Coord, Size};
use prototty_title::*;

// The `Border` decorator in prototty_common requires that the
// view which it decorates implements `ViewSize`, defined in
// prototty. Since neither `ViewSize` nor `Title` are defined
// in this crate, we need to define a new type, and implement
// `View` and `ViewSize` here in this crate.
struct SizedDemoTitleView;
impl View<Title> for SizedDemoTitleView {
    fn view<G: ViewGrid>(&self, title: &Title, coord: Coord, depth: i32, grid: &mut G) {
        // behave identically to `DemoTitleView`
        DemoTitleView.view(title, coord, depth, grid);
    }
}
impl ViewSize<Title> for SizedDemoTitleView {
    fn size(&self, title: &Title) -> Size {
        // 3 high, since the title is rendered on
        // line 0 and 2
        Size::new(title.width, 3)
    }
}

fn main() {

    let mut context = prototty_unix::Context::new().unwrap();

    let title = Title {
        width: 20,
        text: "My Title".to_string(),
    };

    // create and configure the border
    let mut border = prototty_common::Border::new();
    border.title = Some("Border Title".to_string());
    border.underline_title = true;

    // create a decorated view
    let decorated_view = prototty_common::Decorated::new(&SizedDemoTitleView, &border);

    // render the title using the view
    context.render(&decorated_view, &title).unwrap();

    // exit after a key is pressed
    context.wait_input().unwrap();
}
