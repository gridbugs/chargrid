use chargrid_core::*;

#[derive(Debug, Clone, Copy)]
pub enum AlignmentX {
    Left,
    Centre,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum AlignmentY {
    Top,
    Centre,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub struct Alignment {
    pub x: AlignmentX,
    pub y: AlignmentY,
}

impl Alignment {
    pub fn centre() -> Self {
        Self {
            x: AlignmentX::Centre,
            y: AlignmentY::Centre,
        }
    }
}

pub struct Align<C: Component> {
    pub component: C,
    pub alignment: Alignment,
}

impl<C: Component> Align<C> {
    pub fn centre(component: C) -> Self {
        Self {
            component,
            alignment: Alignment::centre(),
        }
    }
    fn child_ctx<'a>(&self, state: &C::State, ctx: Ctx<'a>) -> Ctx<'a> {
        let size = self.component.size(state, ctx);
        let ctx_size = ctx.bounding_box.size();
        let x_offset = match self.alignment.x {
            AlignmentX::Left => 0,
            AlignmentX::Centre => (ctx_size.x() as i32 - size.x() as i32) / 2,
            AlignmentX::Right => ctx_size.x() as i32 - size.x() as i32,
        };
        let y_offset = match self.alignment.y {
            AlignmentY::Top => 0,
            AlignmentY::Centre => (ctx_size.y() as i32 - size.y() as i32) / 2,
            AlignmentY::Bottom => ctx_size.y() as i32 - size.y() as i32,
        };
        ctx.add_offset(Coord::new(x_offset, y_offset))
    }
}

impl<C: Component> Component for Align<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.component.render(state, self.child_ctx(state, ctx), fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component
            .update(state, self.child_ctx(state, ctx), event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, self.child_ctx(state, ctx))
    }
}
