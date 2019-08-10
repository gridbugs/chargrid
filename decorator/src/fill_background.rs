use prototty_render::*;

pub struct FillBackgroundView_<V> {
    pub view: V,
    pub rgb24: Rgb24,
}

impl<V, T> View<T> for FillBackgroundView_<V>
where
    V: View<T>,
{
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) {
        let size = self
            .view
            .view_reporting_intended_size(data, context.add_depth(1), frame);
        for y in 0..(size.height() as i32) {
            for x in 0..(size.width() as i32) {
                let coord = Coord::new(x, y);
                frame.set_cell_relative(
                    coord,
                    0,
                    ViewCell::new().with_background(self.rgb24).with_character(' '),
                    context,
                );
            }
        }
    }

    fn visible_bounds<C: ColModify>(&mut self, data: T, context: ViewContext<C>) -> Size {
        self.view.visible_bounds(data, context)
    }
}
