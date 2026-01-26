use chargrid_core::*;

pub struct PadTo<C: Component> {
    pub component: C,
    pub size: UCoord,
}

impl<C: Component> Component for PadTo<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> UCoord {
        self.component.size(state, ctx).pairwise_max(self.size)
    }
}
