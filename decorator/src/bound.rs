use chargrid_render::*;

pub struct BoundView<V> {
    pub view: V,
    pub size: Size,
}

impl<V, T> View<T> for BoundView<V>
where
    V: View<T>,
{
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) {
        self.view.view(data, context.constrain_size_to(self.size), frame);
    }

    fn size<C: ColModify>(&mut self, _: T, _context: ViewContext<C>) -> Size {
        self.size
    }

    fn view_size<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) -> Size {
        self.view(data, context, frame);
        self.size
    }
}
