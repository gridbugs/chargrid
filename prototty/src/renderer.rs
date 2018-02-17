use super::{Coord, Size};
use view::View;

/// Trait to implement for renderers that take data and a view that knows how
/// to render the data to a grid, and actually draws the result.
pub trait Renderer {
    type Error: ::std::fmt::Debug;

    fn render_at<V: View<T>, T>(
        &mut self,
        view: &mut V,
        data: &T,
        offset: Coord,
        depth: i32,
    ) -> Result<(), Self::Error>;
    fn render<V: View<T>, T>(&mut self, view: &mut V, data: &T) -> Result<(), Self::Error> {
        self.render_at(view, data, Coord::new(0, 0), 0)
    }
    fn size(&self) -> Size;
}
