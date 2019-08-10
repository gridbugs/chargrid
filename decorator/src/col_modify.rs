use prototty_render::*;

pub struct ColModifyView_<V, C> {
    pub view: V,
    pub col_modify: C,
}

impl<V, C, T> View<T> for ColModifyView_<V, C>
where
    V: View<T>,
    C: ColModify,
{
    fn view<F: Frame, C1: ColModify>(&mut self, data: T, context: ViewContext<C1>, frame: &mut F) {
        self.view.view(data, context.compose_col_modify(self.col_modify), frame);
    }
}
