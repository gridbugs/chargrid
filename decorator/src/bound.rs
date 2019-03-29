use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct BoundView<V> {
    pub view: V,
}

impl<V> BoundView<V> {
    pub fn new(view: V) -> Self {
        Self { view }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct BoundData<T> {
    pub size: Size,
    pub data: T,
}

impl<'a, T, V: View<&'a T>> View<&'a BoundData<T>> for BoundView<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        &BoundData { size, ref data }: &'a BoundData<T>,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.view(BoundData { size, data }, context, grid);
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        &BoundData { size, ref data }: &'a BoundData<T>,
        context: ViewContext<R>,
    ) -> Size {
        self.visible_bounds(BoundData { size, data }, context)
    }
}

impl<T, V: View<T>> View<BoundData<T>> for BoundView<V> {
    fn view<G: ViewGrid, R: ViewTransformRgb24>(
        &mut self,
        BoundData { size, data }: BoundData<T>,
        context: ViewContext<R>,
        grid: &mut G,
    ) {
        self.view.view(data, context.constrain_size_to(size), grid);
    }
    fn visible_bounds<R: ViewTransformRgb24>(
        &mut self,
        BoundData { size, data: _ }: BoundData<T>,
        _context: ViewContext<R>,
    ) -> Size {
        size
    }
}
