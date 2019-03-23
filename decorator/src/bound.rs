use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct Bounded<V> {
    pub view: V,
    pub size: Size,
}

impl<V> Bounded<V> {
    pub fn new(view: V, size: Size) -> Self {
        Self { view, size }
    }
}

impl<T, V: View<T>> View<T> for Bounded<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        data: T,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.view
            .view(data, context.constrain_size_to(self.size), grid);
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        _data: T,
        _context: ViewContext<R>,
    ) -> Size {
        self.size
    }
}
