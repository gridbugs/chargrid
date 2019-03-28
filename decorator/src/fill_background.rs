use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct FillBackgroundView<V> {
    pub view: V,
}

impl<V> FillBackgroundView<V> {
    pub fn new(view: V) -> Self {
        Self { view }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct FillBackgroundData<T> {
    pub background: Rgb24,
    pub data: T,
}

impl<T, V: View<T>> View<FillBackgroundData<T>> for FillBackgroundView<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        FillBackgroundData { background, data }: FillBackgroundData<T>,
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
                        .with_background(background)
                        .with_character(' '),
                    context,
                );
            }
        }
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        FillBackgroundData {
            background: _,
            data,
        }: FillBackgroundData<T>,
        context: ViewContext<R>,
    ) -> Size {
        self.view.visible_bounds(data, context)
    }
}
