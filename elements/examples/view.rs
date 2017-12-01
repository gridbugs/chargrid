extern crate prototty;
extern crate cgmath;

use prototty::*;
use cgmath::Vector2;

struct Model;
impl View for Model {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        let s = "Hello, World!";
        for (i, c) in s.chars().enumerate() {
            let coord = offset + Vector2::new(i + 1, 1).cast();
            grid.get_mut(coord).map(|cell| cell.update(c, depth));
        }
    }
    fn size(&self) -> Vector2<u16> { Vector2::new(10, 10) }
}

fn main() {
    let mut context = Context::new().unwrap();
    context.render(&Model).unwrap();
    context.wait_input().unwrap();
}
