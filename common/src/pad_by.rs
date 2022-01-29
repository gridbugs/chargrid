use chargrid_core::*;

pub struct Padding {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}
impl Padding {
    pub fn all(padding: u32) -> Self {
        Self {
            top: padding,
            bottom: padding,
            left: padding,
            right: padding,
        }
    }
}

pub struct PadBy<C: Component> {
    pub component: C,
    pub padding: Padding,
}

impl Padding {
    fn update_ctx<'a>(&self, ctx: Ctx<'a>) -> Ctx<'a> {
        ctx.add_offset(Coord::new(self.left as i32, self.top as i32))
            .add_size(Size::new(self.right, self.bottom))
    }
}

impl<C: Component> Component for PadBy<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component
            .render(state, self.padding.update_ctx(ctx), fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component
            .update(state, self.padding.update_ctx(ctx), event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, self.padding.update_ctx(ctx))
    }
}
