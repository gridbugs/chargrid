extern crate prototty_render;
use prototty_render::*;

// Define a type representing the element
pub struct Title {
    pub width: u32,
    pub text: String,
}

// Define a type representing how the element will be rendered.
pub struct TitleView;

// Describe how a TitleView renders a Title by implementing View.
impl<'a> View<&'a Title> for TitleView {
    fn view<F: Frame, R: ViewTransformRgb24>(
        &mut self,
        title: &'a Title,
        context: ViewContext<R>,
        frame: &mut F,
    ) {
        for (i, ch) in title.text.chars().enumerate() {
            frame.set_cell_relative(
                Coord::new(i as i32, 0),
                0,
                ViewCell::new()
                    .with_character(ch)
                    .with_bold(true)
                    .with_foreground(Rgb24::new(0, 0, 0))
                    .with_background(Rgb24::new(0, 255, 127)),
                context,
            );
        }
    }
}

// What if we want a way to rendered titles centered within their width?
pub struct CenteredTitleView;
impl<'a> View<&'a Title> for CenteredTitleView {
    fn view<F: Frame, R: ViewTransformRgb24>(
        &mut self,
        title: &'a Title,
        context: ViewContext<R>,
        frame: &mut F,
    ) {
        let space = ::std::cmp::max(title.width as i32 - title.text.len() as i32, 0) / 2;
        for (i, ch) in title.text.chars().enumerate() {
            frame.set_cell_relative(
                Coord::new(space + i as i32, 0),
                0,
                ViewCell::new()
                    .with_character(ch)
                    .with_underline(true)
                    .with_foreground(Rgb24::new(255, 127, 0)),
                context,
            );
        }
    }
}

// Let's demonstrate both of these in action by rendering a title
// twice - once left aligned, an once centered:
pub struct DemoTitleView;
impl<'a> View<&'a Title> for DemoTitleView {
    fn view<F: Frame, R: ViewTransformRgb24>(
        &mut self,
        title: &'a Title,
        context: ViewContext<R>,
        frame: &mut F,
    ) {
        // render the title left-aligned in the top-left corner
        TitleView.view(title, context, frame);

        // render the title centered 2 lines down
        CenteredTitleView.view(title, context.add_offset(Coord::new(0, 2)), frame);
    }
}
