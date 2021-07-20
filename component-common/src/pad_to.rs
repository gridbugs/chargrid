use chargrid_component::*;

pub struct PadTo<C: Component> {
    pub component: C,
    pub size: Size,
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
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx).pairwise_max(self.size)
    }
}

pub type PurePadTo<C> = convert::ComponentPureT<PadTo<C>>;

impl<C: Component<State = ()>> PadTo<C> {
    pub fn pure(self) -> PurePadTo<C> {
        convert::ComponentPureT(self)
    }
}
