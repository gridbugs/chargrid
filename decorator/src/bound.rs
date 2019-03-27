use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Bound<V>(pub V);

impl<T, V: View<T>> View<(T, Size)> for Bound<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        (data, size): (T, Size),
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.0.view(data, context.constrain_size_to(size), grid);
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        (_data, size): (T, Size),
        _context: ViewContext<R>,
    ) -> Size {
        size
    }
}
