extern crate prototty;
extern crate prototty_unix;

use std::thread;
use std::time::Duration;
use prototty::menu::*;
use prototty::unix::*;
use prototty::traits::*;
use prototty::decorator::*;

const PERIOD_MILLIS: u64 = 16;
const DELAY_MILLIS: u64 = 1000;

struct IntView;
impl View<i32> for IntView {
    fn view<G: ViewGrid>(&self, value: &i32,
                         offset: Coord, depth: i16, grid: &mut G)
    {
        let s = value.to_string();
        for (i, c) in s.chars().enumerate() {
            grid.get_mut(offset + Coord::new(i as i16, 0)).map(|cell| {
                cell.update(c, depth);
            });
        }
    }
}
impl ViewSize<i32> for IntView {
    fn size(&self, _: &i32) -> Size {
        Size::new(1, 1)
    }
}

fn main() {
    let mut context = Context::new().unwrap();

    let menu = Menu::smallest(vec![
        ("Zero", 0),
        ("One", 1),
        ("Two", 2),
        ("Three", 3),
    ]);

    let mut instance = MenuInstance::new(&menu).unwrap();
    let border = Border::new();

    let mut number = None;

    loop {
        if let Some(output) = instance.tick(context.drain_input().unwrap()) {
            match output {
                MenuOutput::Quit | MenuOutput::Cancel => break,
                MenuOutput::Finalise(x) => {
                    number = Some(x);
                }
            }
        }
        if let Some(x) = number {
            context.render(&Decorated::new(&IntView, &border), &x).unwrap();
            number = None;
            thread::sleep(Duration::from_millis(DELAY_MILLIS));
        } else {
            context.render(&Decorated::new(&DefaultMenuInstanceView, &border), &instance).unwrap();
            thread::sleep(Duration::from_millis(PERIOD_MILLIS));
        }
    }
}
