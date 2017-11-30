use cgmath::Vector2;
use ansi_colour::Colour;

pub trait ViewCell {
    fn update(&mut self, ch: char, depth: i16);
    fn update_with_colour(&mut self, ch: char, depth: i16, fg: Colour, bg: Colour);
    fn update_with_style(&mut self, ch: char, depth: i16, fg: Colour, bg: Colour,
                         bold: bool, underline: bool);
}

pub trait ViewGrid {
    type Cell: ViewCell;
    fn get_mut(&mut self, coord: Vector2<i16>) -> Option<&mut Self::Cell>;
}

pub trait View {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G);
}
