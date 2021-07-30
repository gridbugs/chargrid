use chargrid_component::{input::Input, Event};
pub use chargrid_component::{
    Component, ControlFlow, Coord, FrameBuffer, FrameBufferCell, Rgba32, Size,
};
use std::time::Duration;

pub fn on_input<C>(
    component: &mut C,
    input: Input,
    frame_buffer: &FrameBuffer,
) -> Option<ControlFlow>
where
    C: Component<State = (), Output = Option<ControlFlow>>,
{
    component.update(&mut (), frame_buffer.default_ctx(), Event::Input(input))
}

pub fn on_frame<C>(
    component: &mut C,
    since_last_frame: Duration,
    frame_buffer: &mut FrameBuffer,
) -> Option<ControlFlow>
where
    C: Component<State = (), Output = Option<ControlFlow>>,
{
    let ctx = frame_buffer.default_ctx();
    if let Some(control_flow) = component.update(&mut (), ctx, Event::Tick(since_last_frame)) {
        return Some(control_flow);
    }
    component.render(&(), ctx, frame_buffer);
    None
}
