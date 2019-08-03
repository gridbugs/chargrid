use crate::controls::Controls;
use game::{Game, ToRender};
use prototty::event_routine::common_event::*;
use prototty::event_routine::*;
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

pub struct GameData {
    game: Game,
    controls: Controls,
}

impl GameData {
    pub fn new(controls: Controls) -> Self {
        Self {
            game: Game::new(),
            controls,
        }
    }
}

pub struct GameEventRoutine;
impl EventRoutine for GameEventRoutine {
    type Return = ();
    type Data = GameData;
    type View = GameView;
    type Event = CommonEvent;

    fn handle<EP>(self, data: &mut Self::Data, _view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| match event {
            CommonEvent::Input(input) => {
                let maybe_game_input = data.controls.get(input);
                if let Some(game_input) = maybe_game_input {
                    data.game.handle_input(game_input);
                }
                Handled::Continue(s)
            }
            CommonEvent::Frame(_) => Handled::Continue(s),
        })
    }

    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
        view.view(&data.game, context, frame);
    }
}
