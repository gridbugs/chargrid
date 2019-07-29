use prototty_render::*;

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct TransformRgb24View<V> {
    pub view: V,
}

impl<V> TransformRgb24View<V> {
    pub fn new(view: V) -> Self {
        Self { view }
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy)]
pub struct TransformRgb24Data<S, T> {
    pub col_modify: S,
    pub data: T,
}

impl<'a, T, V: View<&'a T>, S: ColModify> View<&'a TransformRgb24Data<S, T>> for TransformRgb24View<V> {
    fn view<F: Frame, C: ColModify>(
        &mut self,
        &TransformRgb24Data {
            col_modify,
            ref data,
        }: &'a TransformRgb24Data<S, T>,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        self.view(TransformRgb24Data { col_modify, data }, context, frame)
    }
    fn visible_bounds<C: ColModify>(
        &mut self,
        &TransformRgb24Data {
            col_modify,
            ref data,
        }: &'a TransformRgb24Data<S, T>,
        context: ViewContext<C>,
    ) -> Size {
        self.visible_bounds(TransformRgb24Data { col_modify, data }, context)
    }
}

impl<T, V: View<T>, S: ColModify> View<TransformRgb24Data<S, T>> for TransformRgb24View<V> {
    fn view<F: Frame, C: ColModify>(
        &mut self,
        TransformRgb24Data { col_modify, data }: TransformRgb24Data<S, T>,
        context: ViewContext<C>,
        frame: &mut F,
    ) {
        self.view
            .view(data, context.compose_col_modify(col_modify), frame);
    }
    fn visible_bounds<C: ColModify>(
        &mut self,
        TransformRgb24Data {
            col_modify: _,
            data,
        }: TransformRgb24Data<S, T>,
        context: ViewContext<C>,
    ) -> Size {
        self.view.visible_bounds(data, context)
    }
}
