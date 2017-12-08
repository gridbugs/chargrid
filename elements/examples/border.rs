extern crate prototty;
extern crate prototty_elements;
extern crate ansi_colour;
extern crate cgmath;

use prototty::*;
use prototty_elements::elements::*;
use ansi_colour::colours;
use cgmath::Vector2;

struct App {
    a: Border<Text>,
    b: Border<Text>,
}


impl View for App {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        self.a.view(offset + Vector2::new(1, 1), depth, grid);
        self.b.view(offset + Vector2::new(self.a.size().x as i16 + 6, 1), depth, grid);
    }
}
impl ViewSize for App {
    fn size(&self) -> Vector2<u16> {
        Vector2::new(30, 20)
    }
}

fn main() {
    let mut context = Context::new().unwrap();

    let mut app = App {
        a: Border::new(Text::new("abcdefghijklmnopqrstuvwxyz", (5, 5))),
        b: Border::with_title(Text::new("abcdefghijklmnopqrstuvwxyz", (5, 5)), "Foo"),
    };

    app.b.title_colour = colours::DARK_GREY;
    app.b.foreground_colour = colours::RED;
    app.b.background_colour = colours::BLUE;

    app.a.bold_border = true;
    app.b.bold_title = true;

    app.a.chars.top_left = '#';
    app.a.chars.bottom_left = '#';
    app.a.chars.top_right = '#';
    app.a.chars.bottom_right = '#';
    app.a.chars.left = '#';
    app.a.chars.right = '#';
    app.a.chars.top = '#';
    app.a.chars.bottom = '#';

    app.a.padding.top = 1;
    app.a.padding.left = 2;
    app.a.padding.right = 2;
    app.a.padding.bottom = 4;

    let mut app = Border::with_title(app, "Hello, World!");
    app.underline_title = true;

    context.render(&app).unwrap();
    context.wait_input().unwrap();
}
