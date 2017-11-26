extern crate prototty;
extern crate cgmath;

use std::thread;
use std::time::{Instant, Duration};
use prototty::*;
use prototty::elements::*;

const NUM_FRAMES: u32 = 4000;

fn main() {

    let div = AbsDiv::new((20, 10));
    let root_element = ElementHandle::from(div.clone());

    let counter = Text::new("", (10, 1));
    div.insert("counter", counter.clone(), (0, 0), None);

    let duration = {
        let mut ctx = Context::new().unwrap();

        let start = Instant::now();
        for i in 0..NUM_FRAMES {
            counter.set(format!("{}", i));
            ctx.render(&root_element).unwrap();

            thread::sleep(Duration::from_millis(1));
        }

        let end = Instant::now();

        end - start
    };

    let average_per_frame = duration / NUM_FRAMES;
    let secs_per_frame = average_per_frame.as_secs() as f64 +
        average_per_frame.subsec_nanos() as f64 * 1e-9;
    let average_fps = 1.0 / secs_per_frame;

    println!("Frames: {}", NUM_FRAMES);
    println!("Total time: {:?}", duration);
    println!("Average per frame: {:?}", average_per_frame);
    println!("Average FPS: {:?}", average_fps);
}
