use prototty_web::{Context, Size, WebAudioPlayer};
use soundboard_prototty as app;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    console_error_panic_hook::set_once();
    let player = WebAudioPlayer::new_with_mime("video/ogg");
    Context::new(Size::new(16, 8), "content").run_event_routine_repeating(
        app::event_routine(),
        app::AppData::new(player),
        app::AppView::new(),
        |_| app::event_routine(),
    );
    Ok(())
}
