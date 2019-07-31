use game::{Game, ToRender};
use prototty::render::*;

pub struct GameView;

impl<'a> View<&'a Game> for GameView {
    fn view<F: Frame, C: ColModify>(&mut self, game: &'a Game, context: ViewContext<C>, frame: &mut F) {
        let ToRender { grid } = game.to_render();
        for (coord, cell) in grid.enumerate() {
            let character = match cell.occupant {
                None => '.',
                Some(game::Occupant::Player) => '@',
                Some(game::Occupant::Wall) => '#',
            };
            let view_cell = ViewCell::new().with_character(character);
            frame.set_cell_relative(coord, 0, view_cell, context);
        }
    }
}
