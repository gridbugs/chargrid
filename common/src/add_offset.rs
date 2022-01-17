use chargrid_core::*;

pub struct AddOffset<C: Component> {
    pub component: C,
    pub offset: Coord,
}

impl<C: Component> Component for AddOffset<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        let ctx = ctx.add_offset(self.offset);
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        let ctx = ctx.add_offset(self.offset);
        self.component.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        let ctx = ctx.add_offset(self.offset);
        self.component.size(state, ctx)
    }
}
