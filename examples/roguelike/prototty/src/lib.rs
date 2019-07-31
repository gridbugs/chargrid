use event_routine::*;
use game::{Direction, Game};
use prototty::*;
use render::View;

mod game_view;

pub struct AppData {
    game: Game,
}

impl AppData {
    pub fn new() -> Self {
        Self { game: Game::new() }
    }
}

pub struct AppView {
    game: game_view::GameView,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            game: game_view::GameView,
        }
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

    fn view<F, C>(&self, data: &Self::Data, view: &mut Self::View, context: ViewContext<C>, frame: &mut F)
    where
        F: Frame,
        C: ColModify,
    {
        view.game.view(&data.game, context, frame);
    }
}

pub fn event_routine() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    GameEventRoutine
}
