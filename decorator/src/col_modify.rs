use chargrid_render::*;

pub struct ColModifyView<V, C> {
    pub view: V,
    pub col_modify: C,
}

impl<V, C, T> View<T> for ColModifyView<V, C>
where
    V: View<T>,
    C: ColModify,
{
    fn view<F: Frame, C1: ColModify>(&mut self, data: T, context: ViewContext<C1>, frame: &mut F) {
        self.view.view(data, context.compose_col_modify(self.col_modify), frame);
    }
}
