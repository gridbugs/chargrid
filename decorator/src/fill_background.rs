use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct FillBackground<V>(pub V);

impl<T, V: View<T>> View<(T, Rgb24)> for FillBackground<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        (data, rgb24): (T, Rgb24),
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let size = self
            .0
            .view_reporting_intended_size(data, context.add_depth(1), grid);
        for y in 0..(size.height() as i32) {
            for x in 0..(size.width() as i32) {
                let coord = Coord::new(x, y);
                grid.set_cell_relative(
                    coord,
                    0,
                    ViewCell::new().with_background(rgb24).with_character(' '),
                    context,
                );
            }
        }
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        (data, _rgb24): (T, Rgb24),
        context: ViewContext<R>,
    ) -> Size {
        self.0.visible_bounds(data, context)
    }
}
