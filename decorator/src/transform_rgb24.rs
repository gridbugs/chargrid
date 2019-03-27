use prototty_render::*;

pub struct TransformRgb24<V>(pub V);

impl<T, V: View<T>, S: ViewTransformRgb24> View<(T, S)> for TransformRgb24<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        (data, transform_rgb24): (T, S),
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.0
            .view(data, context.compose_transform_rgb24(transform_rgb24), grid);
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        (data, _transform_rgb24): (T, S),
        context: ViewContext<R>,
    ) -> Size {
        self.0.visible_bounds(data, context)
    }
}
