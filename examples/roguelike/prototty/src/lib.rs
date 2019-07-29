use event_routine::*;
use game::{Direction, Game, ToRender};
use prototty::*;
use render::ViewCell;

pub struct AppData {
    game: Game,
}

impl AppData {
    pub fn new() -> Self {
        Self { game: Game::new() }
    }
}

pub struct AppView {}

impl AppView {
    pub fn new() -> Self {
        Self {}
    }
}

struct GameEventRoutine;
impl EventRoutine for GameEventRoutine {
    type Return = ();
    type Data = AppData;
    type View = AppView;
    type Event = CommonEvent;

    fn handle<EP>(self, data: &mut Self::Data, _view: &Self::View, event_or_peek: EP) -> Handled<Self::Return, Self>
    where
        EP: EventOrPeek<Event = Self::Event>,
    {
        event_or_peek_with_handled(event_or_peek, self, |s, event| match event {
            CommonEvent::Input(input) => {
                let maybe_game_input = match input {
                    input::Input::Left => Some(game::Input::Move(Direction::West)),
                    input::Input::Right => Some(game::Input::Move(Direction::East)),
                    input::Input::Up => Some(game::Input::Move(Direction::North)),
                    input::Input::Down => Some(game::Input::Move(Direction::South)),
                    input::inputs::ETX => return Handled::Return(()),
                    _ => None,
                };
                if let Some(game_input) = maybe_game_input {
                    data.game.handle_input(game_input);
                }
                Handled::Continue(s)
            }
            CommonEvent::Frame(_) => Handled::Continue(s),
        })
    }

    fn view<F, R>(&self, data: &Self::Data, _view: &mut Self::View, context: ViewContext<R>, frame: &mut F)
    where
        F: Frame,
        R: ViewTransformRgb24,
    {
        let ToRender { grid } = data.game.to_render();
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

pub fn event_routine() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    GameEventRoutine
}
