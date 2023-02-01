use chargrid::{control_flow::*, core::*, text};

pub fn app() -> impl Component<State = (), Output = app::Output> {
    cf(text::StyledString::plain_text("hello".to_string()))
        .ignore_output()
        .exit_on_close()
}
