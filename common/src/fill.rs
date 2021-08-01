use chargrid_core::*;

pub struct Fill<C: Component> {
    pub component: C,
    pub background: Rgba32,
}

impl<C: Component> Component for Fill<C> {
    type Output = C::Output;
    type State = C::State;
    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        let size = self.component.size(state, ctx);
        for y in 0..(size.height() as i32) {
            for x in 0..(size.width() as i32) {
                let coord = Coord::new(x, y);
                fb.set_cell_relative_to_ctx(
                    ctx,
                    coord,
                    0,
                    RenderCell {
                        style: Style {
                            background: Some(self.background),
                            ..Default::default()
                        },
                        character: Some(' '),
                    },
                );
            }
        }
        self.component.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.component.update(state, ctx, event)
    }
    fn size(&self, state: &Self::State, ctx: Ctx) -> Size {
        self.component.size(state, ctx)
    }
}
