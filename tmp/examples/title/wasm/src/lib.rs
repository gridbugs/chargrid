extern crate prototty;
extern crate prototty_wasm;

// Assuming the title and its views were defined here
extern crate prototty_title;

use prototty::Renderer;
use prototty_title::*;

// Define a type containing the entire application state.
pub struct App {
    title: Title,
    context: prototty_wasm::Context,
}

// Implement a function "alloc_app", which allocates the
// application state, returning a pointer to it. This will
// be called by the prototty-terminal-js library.
//
// This function takes a rng seed, which we ignore here.
#[no_mangle]
pub extern "C" fn alloc_app(_seed: usize) -> *mut App {
    let context = prototty_wasm::Context::new();
    let title = Title {
        width: 20,
        text: "My Title".to_string(),
    };
    let app = App { title, context };

    prototty_wasm::alloc::into_boxed_raw(app)
}

// Implement a function "tick", which is called periodically
// by prototty-terminal-js. It's passed a pointer to the app
// state (allocated by "alloc_app"), and some information about
// inputs and the time that passed since it was last called,
// which we ignore here.
#[no_mangle]
pub unsafe fn tick(app: *mut App,
                   _key_codes: *const u8,
                   _key_mods: *const u8,
                   _num_inputs: usize,
                   _period_millis: f64) {

    (*app).context.render(&DemoTitleView, &(*app).title).unwrap();
}
