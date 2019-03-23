use prototty_render::*;

pub struct Identity<V>(V);

impl<V> Identity<V> {
    pub fn new(view: V) -> Self {
        Self(view)
    }
}

impl<T, V: View<T>> View<T> for Identity<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.0.view(data, context, grid);
    }

    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
    ) -> Size {
        self.0.visible_bounds(data, context)
    }

    fn view_reporting_intended_size<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        grid: &mut G,
    ) -> Size {
        self.0.view_reporting_intended_size(data, context, grid)
    }
}
