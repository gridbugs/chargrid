use prototty::*;
use menu::MenuInstance;

const ESCAPE: char = '\u{1b}';
const ETX: char = '\u{3}';
const RETURN: char = '\u{d}';

#[derive(Debug, Clone, Copy)]
pub enum MenuAction<T> {
    Quit,
    Escape,
    Select(T),
}

pub trait MenuRunner {
    fn run_menu<T, V, F>(&mut self, view: &mut V, get_instance: F) -> Result<MenuAction<T>>
        where T: Copy,
              V: View,
              F: Fn(&mut V) -> &mut MenuInstance<T>;

}

impl MenuRunner for Context {
    fn run_menu<T, V, F>(&mut self, view: &mut V, get_instance: F) -> Result<MenuAction<T>>
        where T: Copy,
              V: View,
              F: Fn(&mut V) -> &mut MenuInstance<T>
    {
        loop {
            self.render(view)?;
            match self.wait_input()? {
                Input::Char(ETX) => return Ok(MenuAction::Quit),
                Input::Char(ESCAPE) => return Ok(MenuAction::Escape),
                Input::Char(RETURN) => {
                    return Ok(MenuAction::Select(get_instance(view).selected()));
                }
                Input::Up => get_instance(view).up(),
                Input::Down => get_instance(view).down(),
                _ => (),
            }
        }
    }
}
