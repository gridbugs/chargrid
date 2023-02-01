use chargrid::{border::*, control_flow::*, core::*, text_field::TextField};

struct AppComponent {
    text_field: Border<TextField>,
}

impl AppComponent {
    fn new() -> Self {
        Self {
            text_field: Border {
                component: TextField::with_initial_string(12, "test".to_string()),
                style: BorderStyle::default(),
            },
        }
    }
}

impl Component for AppComponent {
    type Output = ();
    type State = ();

    fn render(&self, state: &Self::State, ctx: Ctx, fb: &mut FrameBuffer) {
        self.text_field.render(state, ctx, fb);
    }
    fn update(&mut self, state: &mut Self::State, ctx: Ctx, event: Event) -> Self::Output {
        self.text_field.update(state, ctx, event);
    }
    fn size(&self, _state: &Self::State, ctx: Ctx) -> Size {
        ctx.bounding_box.size()
    }
}

pub fn app() -> impl Component<State = (), Output = app::Output> {
    cf(AppComponent::new())
        .ignore_output()
        .exit_on_close()
        .clear_each_frame()
}
