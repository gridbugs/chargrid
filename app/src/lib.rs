pub use chargrid_input::Input;
pub use chargrid_render::{ColModify, Frame, ViewContext};
pub use std::time::Duration;

pub enum ControlFlow {
    Exit,
}

pub trait App {
    fn on_input(&mut self, input: Input) -> Option<ControlFlow>;
    fn on_frame<F, C>(
        &mut self,
        since_last_frame: Duration,
        view_context: ViewContext<C>,
        frame: &mut F,
    ) -> Option<ControlFlow>
    where
        F: Frame,
        C: ColModify;
}
