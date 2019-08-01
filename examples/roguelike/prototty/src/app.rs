use crate::controls::Controls;
use crate::game_view;
use common_event::*;
use event_routine::*;
use game::Game;
use prototty::*;
use render::View;

pub struct AppData {
    game: Game,
    controls: Controls,
}

impl AppData {
    pub fn new() -> Self {
        Self {
            game: Game::new(),
            controls: Controls::default(),
        }
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
        view.game.view(&data.game, context, frame);
    }
}

pub fn event_routine() -> impl EventRoutine<Return = (), Data = AppData, View = AppView, Event = CommonEvent> {
    GameEventRoutine.return_on_exit(|_| ())
}
