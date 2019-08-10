use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct ColModifyView<V> {
    pub view: V,
}

impl<V> ColModifyView<V> {
    pub fn new(view: V) -> Self {
        Self { view }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct ColModifyData<S, T> {
    pub col_modify: S,
    pub data: T,
}

impl<'a, T, V: View<&'a T>, S: ColModify> View<&'a ColModifyData<S, T>> for ColModifyView<V> {
    fn view<F: Frame, C: ColModify>(
        &mut self,
        &ColModifyData { col_modify, ref data }: &'a ColModifyData<S, T>,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        self.view(ColModifyData { col_modify, data }, context, frame)
    }
    fn visible_bounds<C: ColModify>(
        &mut self,
        &ColModifyData { col_modify, ref data }: &'a ColModifyData<S, T>,
        context: ViewContext<C>,
    ) -> Size {
        self.visible_bounds(ColModifyData { col_modify, data }, context)
    }
}

impl<T, V: View<T>, S: ColModify> View<ColModifyData<S, T>> for ColModifyView<V> {
    fn view<F: Frame, C: ColModify>(
        &mut self,
        ColModifyData { col_modify, data }: ColModifyData<S, T>,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        self.view.view(data, context.compose_col_modify(col_modify), frame);
    }
    fn visible_bounds<C: ColModify>(
        &mut self,
        ColModifyData { col_modify: _, data }: ColModifyData<S, T>,
        context: ViewContext<C>,
    ) -> Size {
        self.view.visible_bounds(data, context)
    }
}

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
