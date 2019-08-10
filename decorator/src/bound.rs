use prototty_render::*;

pub struct BoundView_<V> {
    pub view: V,
    pub size: Size,
}

impl<V, T> View<T> for BoundView_<V>
where
    V: View<T>,
{
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) {
        self.view.view(data, context.constrain_size_to(self.size), frame);
    }

    fn visible_bounds<C: ColModify>(&mut self, _: T, _context: ViewContext<C>) -> Size {
        self.size
    }

    fn view_reporting_intended_size<F: Frame, C: ColModify>(
        &mut self,
        data: T,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> Size {
        self.view(data, context, frame);
        self.size
    }
}
