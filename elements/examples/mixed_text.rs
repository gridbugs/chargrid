extern crate prototty;
extern crate prototty_elements;
extern crate cgmath;
extern crate ansi_colour;

use ansi_colour::colours;
use cgmath::Vector2;
use prototty::*;
use prototty_elements::*;

struct Model {
    rich_text: RichText,
    plain_text: Text,
}

impl View for Model {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        self.rich_text.view(offset, depth, grid);
        self.plain_text.view(offset + Vector2::new(20, 0), depth, grid);
    }
}

fn main() {

    let model = Model {
        rich_text: RichText::one_line(vec![("Some red text...", TextInfo::default().foreground_colour(colours::RED))]),
        plain_text: Text::one_line("Some plain text..."),
    };

    let mut ctx = Context::new().unwrap();

    ctx.render(&model).unwrap();
    ctx.wait_input().unwrap();
}
