pub use chargrid_core::{
    app, rgb_int, Component, Coord, FrameBuffer, FrameBufferCell, Rgba32, Size,
};
use chargrid_core::{input::Input, Event};
use std::time::Duration;

pub fn on_input<C>(component: &mut C, input: Input, frame_buffer: &FrameBuffer) -> app::Output
where
    C: Component<State = (), Output = app::Output>,
{
    component.update(&mut (), frame_buffer.default_ctx(), Event::Input(input))
}

pub fn on_frame<C>(
    component: &mut C,
    since_last_frame: Duration,
    frame_buffer: &mut FrameBuffer,
) -> app::Output
where
    C: Component<State = (), Output = app::Output>,
{
    let ctx = frame_buffer.default_ctx();
    if let Some(output) = component.update(&mut (), ctx, Event::Tick(since_last_frame)) {
        return Some(output);
    }
    component.render(&(), ctx, frame_buffer);
    None
}
