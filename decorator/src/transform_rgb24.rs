use prototty_render::*;

pub struct TransformedRgb24<V, S> {
    pub view: V,
    pub transform_rgb24: S,
}

impl<V, S> TransformedRgb24<V, S> {
    pub fn new(view: V, transform_rgb24: S) -> Self {
        Self {
            view,
            transform_rgb24,
        }
    }
}

impl<T, V: View<T>, S: ViewTransformRgb24> View<T> for TransformedRgb24<V, S> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.view.view(
            data,
            context.compose_transform_rgb24(self.transform_rgb24),
            grid,
        );
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
    ) -> Size {
        self.view.visible_bounds(data, context)
    }
}
