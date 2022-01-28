use chargrid_core::*;

pub struct SetSize<C: Component> {
    pub component: C,
    pub size: Size,
}

impl<C: Component> Component for SetSize<C> {
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
    fn size(&self, _state: &Self::State, _ctx: Ctx) -> Size {
        self.size
    }
}

pub struct SetWidth<C: Component> {
    pub component: C,
    pub width: u32,
}

impl<C: Component> Component for SetWidth<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        let ctx = ctx.set_width(self.width);
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        let ctx = ctx.set_width(self.width);
        self.component.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        let ctx = ctx.set_width(self.width);
        self.component.size(state, ctx)
    }
}

pub struct SetHeight<C: Component> {
    pub component: C,
    pub height: u32,
}

impl<C: Component> Component for SetHeight<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        let ctx = ctx.set_height(self.height);
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        let ctx = ctx.set_height(self.height);
        self.component.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        let ctx = ctx.set_height(self.height);
        self.component.size(state, ctx)
    }
}
