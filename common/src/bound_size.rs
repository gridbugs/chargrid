use chargrid_core::*;

pub struct BoundSize<C: Component> {
    pub component: C,
    pub size: Size,
}

impl<C: Component> Component for BoundSize<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        let ctx = ctx.set_size(self.size);
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        let ctx = ctx.set_size(self.size);
        self.component.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        let child_size = self.component.size(state, ctx);
        child_size.pairwise_min(self.size)
    }
}
