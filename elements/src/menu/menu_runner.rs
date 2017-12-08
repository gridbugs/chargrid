use prototty::*;
use menu::MenuInstance;

const ESCAPE: char = '\u{1b}';
const ETX: char = '\u{3}';
const RETURN: char = '\u{d}';

/// The choice made by a user when running a menu.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MenuChoice<T> {
    Quit,
    Cancel,
    Finalise(T),
}

/// Running a menu is the process of displaying the menu with
/// a visible selection, and capturing input to change the selection.
/// It's possbile for a user to submit their selection (usually by
/// pressing a key), to cancel the menu (usually by pressing a
/// different key), or to quit the menu (usually by senting ETX).
pub trait MenuRunner {
    fn run_menu<T, V, F>(&mut self, view: &mut V, get_instance: F) -> Result<MenuChoice<T>>
        where T: Copy,
              V: View,
              F: Fn(&mut V) -> &mut MenuInstance<T>;

}

impl MenuRunner for Context {
    fn run_menu<T, V, F>(&mut self, view: &mut V, get_instance: F) -> Result<MenuChoice<T>>
        where T: Copy,
              V: View,
              F: Fn(&mut V) -> &mut MenuInstance<T>
    {
        loop {
            self.render(view)?;
            match self.wait_input()? {
                Input::Char(ETX) => return Ok(MenuChoice::Quit),
                Input::Char(ESCAPE) => return Ok(MenuChoice::Cancel),
                Input::Char(RETURN) => {
                    return Ok(MenuChoice::Finalise(get_instance(view).selected()));
                }
                Input::Up => get_instance(view).up(),
                Input::Down => get_instance(view).down(),
                _ => (),
            }
        }
    }
}
