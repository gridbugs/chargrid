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
        let child_size = self.component.size(state, ctx.set_size(self.size));
        child_size.pairwise_min(self.size)
    }
}

pub struct BoundWidth<C: Component> {
    pub component: C,
    pub width: u32,
}

impl<C: Component> Component for BoundWidth<C> {
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
        let child_size = self.component.size(state, ctx.set_width(self.width));
        child_size.set_width(child_size.width().min(self.width))
    }
}

pub struct BoundHeight<C: Component> {
    pub component: C,
    pub height: u32,
}

impl<C: Component> Component for BoundHeight<C> {
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
        let child_size = self.component.size(state, ctx.set_height(self.height));
        child_size.set_height(child_size.height().min(self.height))
    }
}
