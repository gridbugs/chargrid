use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct FilledBackground<V> {
    pub view: V,
    pub rgb24: Rgb24,
}

impl<V> FilledBackground<V> {
    pub fn new(view: V, rgb24: Rgb24) -> Self {
        Self { view, rgb24 }
    }
}

impl<T, V: View<T>> View<T> for FilledBackground<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        let size =
            self.view
                .view_reporting_intended_size(data, context.add_depth(1), grid);
        for y in 0..(size.height() as i32) {
            for x in 0..(size.width() as i32) {
                let coord = Coord::new(x, y);
                grid.set_cell_relative(
                    coord,
                    0,
                    ViewCell::new()
                        .with_background(self.rgb24)
                        .with_character(' '),
                    context,
                );
            }
        }
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
    ) -> Size {
        self.view.visible_bounds(data, context)
    }
}
