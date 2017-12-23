extern crate tetris;
extern crate prototty;
extern crate prototty_elements;
extern crate rand;
extern crate prototty_renderer;
extern crate ansi_colour;
extern crate cgmath;

use std::time::Duration;
use std::thread;
use rand::Rng;
use cgmath::Vector2;
use prototty::{Context, Input as ProtottyInput, View, ViewGrid};
use prototty_elements::elements::*;
use prototty_elements::menu::*;
use prototty_elements::common::TextInfo;
use tetris::{Tetris, Input, Meta};
use prototty_renderer::Model;
use ansi_colour::colours;

const TICK_MILLIS: u64 = 16;
const DEATH_ANIMATION_MILLIS: u64 = 500;
const ESCAPE: char = '\u{1b}';
const ETX: char = '\u{3}';

#[derive(Debug, Clone, Copy)]
enum MainMenuChoice {
    Play,
    Quit,
}

type MainMenu = Border<MenuInstance<MainMenuChoice>>;

fn make_menu() -> MainMenu {
    Border::new(MenuInstance::new(
            Menu::smallest(vec![
                           ("Play", MainMenuChoice::Play),
                           ("Quit", MainMenuChoice::Quit),
            ])).unwrap())
}

fn make_end_text() -> RichText {
    let info = TextInfo::default().bold().foreground_colour(colours::RED);
    RichText::one_line(vec![("YOU DIED", info)])
}

fn main_menu(context: &mut Context) -> MainMenuChoice {
    let mut menu =
        Border::new(MenuInstance::new(
                Menu::smallest(vec![
                               ("Play", MainMenuChoice::Play),
                               ("Quit", MainMenuChoice::Quit),
                ])).unwrap());

    match context.run_menu(&mut menu, |b| &mut b.child).unwrap() {
        MenuChoice::Finalise(x) => x,
        _ => MainMenuChoice::Quit,
    }
}

enum AppState {
    Menu,
    Game,
    GameOver,
    EndText,
}

struct App {
    menu: MainMenu,
    game: Model,
    end_text: RichText,
    state: AppState,
    tetris: Tetris,
    timeout: Duration,
}

impl View for App {
    fn view<G: ViewGrid>(&self, offset: Vector2<i16>, depth: i16, grid: &mut G) {
        match self.state {
            AppState::Menu => self.menu.view(offset, depth, grid),
            AppState::Game | AppState::GameOver => self.game.view(offset, depth, grid),
            AppState::EndText => self.end_text.view(offset, depth, grid),
        }
    }
}

enum ControlFlow {
    Exit,
}

impl App {
    fn new<R: Rng>(rng: &mut R) -> Self {
        let tetris = Tetris::new(rng);
        let size = tetris.size();
        let mut model = Model::new(size.x, size.y);
        model.render(&tetris);
        Self {
            menu: make_menu(),
            end_text: make_end_text(),
            game: model,
            state: AppState::Menu,
            tetris,
            timeout: Duration::from_millis(0),
        }
    }
    pub fn tick<I, R>(&mut self, inputs: I, period: Duration, rng: &mut R) -> Option<ControlFlow>
        where I: Iterator<Item=Input>,
              R: Rng,
    {
        match self.state {
            AppState::Menu => {

            }
            _ => {}
        }
        None
    }
}

fn main() {
    let mut context = Context::new().unwrap();
    let mut rng = rand::thread_rng();
    let mut app = App::new(&mut rng);
    let mut input_buf = Vec::with_capacity(8);

    loop {

        context.render(&app).unwrap();
        thread::sleep(Duration::from_millis(TICK_MILLIS));
/*
        while let Some(input) = context.poll_input().unwrap() {
            let input = match input {
                ProtottyInput::Up => Input::Up,
                ProtottyInput::Down => Input::Down,
                ProtottyInput::Left => Input::Left,
                ProtottyInput::Right => Input::Right,
                ProtottyInput::Char(ESCAPE) => break 'game,
                ProtottyInput::Char(ETX) => break 'menu,
                _ => continue,
            };
            input_buf.push(input);
        } */

        if let Some(control_flow) = app.tick(input_buf.drain(..), Duration::from_millis(TICK_MILLIS), &mut rng) {
            match control_flow {
                ControlFlow::Exit => break,
            }
        }
/*
        if let Some(meta) = tetris.tick(input_buf.drain(..), Duration::from_millis(TICK_MILLIS), &mut rng) {
            match meta {
                Meta::GameOver => {
                    model.render(&tetris);
                    context.render(&model).unwrap();
                    thread::sleep(Duration::from_millis(DEATH_ANIMATION_MILLIS));
                    let info = TextInfo::default().bold().foreground_colour(colours::RED);
                    context.render(&RichText::one_line(vec![("YOU DIED", info)])).unwrap();
                    thread::sleep(Duration::from_millis(DEATH_ANIMATION_MILLIS));
                    break 'game;
                }
            }
        }*/
    }
}
