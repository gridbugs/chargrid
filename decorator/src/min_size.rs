use chargrid_render::*;

pub struct MinSizeView<V> {
    pub view: V,
    pub size: Size,
}

impl<V, T> View<T> for MinSizeView<V>
where
    V: View<T>,
{
    fn view<F: Frame, C: ColModify>(&mut self, data: T, context: ViewContext<C>, frame: &mut F) {
        self.view.view(data, context, frame);
    }

    fn size<C: ColModify>(&mut self, data: T, context: ViewContext<C>) -> Size {
        self.view.size(data, context).pairwise_max(self.size)
    }

    fn view_size<F: Frame, C: ColModify>(
        &mut self,
        data: T,
        context: ViewContext<C>,
        frame: &mut F,
    ) -> Size {
        self.view
            .view_size(data, context, frame)
            .pairwise_max(self.size)
    }
}
