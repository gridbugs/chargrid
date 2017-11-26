use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use cgmath::Vector2;
use core::terminal::Terminal;
use input::Input;
use error::{Error, Result};
use grid::*;
use defaults::*;
use elements::*;

const ESCAPE: char = '\u{1b}';
const ETX: char = '\u{3}';
const RETURN: char = '\u{d}';

pub struct Context {
    terminal: Terminal,
    seq: u64,
    grid: Grid<Cell>,
    output_grid: Grid<OutputCell>,
}

impl Context {
    pub fn new() -> Result<Self> {
        Terminal::new().and_then(Self::from_terminal)
    }

    pub fn from_terminal(terminal: Terminal) -> Result<Self> {

        let size = terminal.size()?;
        let grid = Grid::new(size);
        let output_grid = Grid::new(size);

        Ok(Self {
            terminal,
            output_grid,
            grid,
            seq: 0,
        })
    }

    fn resize_if_necessary(&mut self) -> Result<()> {
        let size = self.terminal.size()?;
        if size != self.grid.size() {
            self.grid.resize(size);
            self.output_grid.resize(size);
        }

        Ok(())
    }

    pub fn render(&mut self, root: &ElementHandle) -> Result<()> {
        self.resize_if_necessary()?;
        self.seq += 1;

        self.grid.clear();
        root.render(&mut self.grid, self.seq, Vector2::new(0, 0), 0);
        self.send_grid_contents()?;
        self.terminal.flush_buffer()?;

        Ok(())
    }

    fn send_grid_contents(&mut self) -> Result<()> {

        self.terminal.set_cursor(Vector2::new(0, 0))?;

        let mut bold = false;
        let mut underline = false;
        let mut fg = DEFAULT_FG;
        let mut bg = DEFAULT_BG;
        self.terminal.set_foreground_colour(fg);
        self.terminal.set_background_colour(bg);

        let mut must_move_cursor = false;

        for ((coord, cell), output_cell) in
            izip!(self.grid.enumerate(), self.output_grid.iter_mut())
        {
            if output_cell.matches(cell) {
                must_move_cursor = true;
                continue;
            }

            let reset = if cell.bold != bold {
                if cell.bold {
                    self.terminal.set_bold();
                    bold = true;
                    false
                } else {
                    self.terminal.reset();
                    bold = false;
                    true
                }
            } else {
                false
            };

            if reset || cell.fg != fg {
                self.terminal.set_foreground_colour(cell.fg);
                fg = cell.fg;
            }

            if reset || cell.bg != bg {
                self.terminal.set_background_colour(cell.bg);
                bg = cell.bg;
            }

            if reset || (cell.underline != underline) {
                if cell.underline {
                    self.terminal.set_underline();
                } else {
                    self.terminal.clear_underline();
                }
                underline = cell.underline;
            }

            if must_move_cursor {
                self.terminal.set_cursor(coord.cast())?;
                must_move_cursor = false;
            }

            output_cell.copy_fields(cell);
            self.terminal.add_char_to_buffer(cell.ch);
        }

        Ok(())
    }

    pub fn run_menu<'a, T>(&mut self, place_name: &str, choices: &'a MenuChoices<T>, root: &ElementHandle) -> Result<MenuSelection<'a, T>> {
        let menu = if let Some(menu) = root.find_menu_place(place_name) {
            menu
        } else {
            return Err(Error::NoSuchMenuPlace(place_name.to_string()));
        };

        let mut index = 0;

        let selection = loop {
            menu.set_menu(choices.strings(), index);
            self.render(root)?;
            match self.wait_input()? {
                Input::Char(ETX) => break MenuSelection::Etx,
                Input::Char(ESCAPE) => break MenuSelection::Escape,
                Input::Char(RETURN) => {
                    break MenuSelection::Selection(&choices.choices[index].value);
                }
                Input::Up => {
                    index = index.saturating_sub(1);
                }
                Input::Down => {
                    if index + 1 < choices.choices.len() {
                        index += 1;
                    }
                }
                _ => {}
            }
        };

        menu.clear_menu();
        Ok(selection)
    }

    pub fn wait_input(&mut self) -> Result<Input> {
        self.terminal.wait_input()
    }
    pub fn wait_input_timeout(&mut self, timeout: Duration) -> Result<Option<Input>> {
        self.terminal.wait_input_timeout(timeout)
    }
    pub fn poll_input(&mut self) -> Result<Option<Input>> {
        self.terminal.poll_input()
    }
}

pub(crate) type ElementCell<T> = Rc<RefCell<T>>;
pub(crate) fn element_cell<T>(t: T) -> ElementCell<T> {
    Rc::new(RefCell::new(t))
}

pub enum MenuSelection<'a, T: 'a> {
    Selection(&'a T),
    Escape,
    Etx,
}

impl<'a, T> MenuSelection<'a, T> {
    pub fn selection(&self) -> Option<&T> {
        match self {
            &MenuSelection::Selection(t) => Some(t),
            _ => None,
        }
    }
}

struct MenuChoice<T> {
    text: String,
    value: T,
}

pub struct MenuChoices<T> {
    choices: Vec<MenuChoice<T>>,
}

impl<T> MenuChoices<T> {
    pub fn new<S: Into<String>>(mut choices: Vec<(S, T)>) -> Self {
        Self {
            choices: choices.drain(..).map(|(s, t)| {
                MenuChoice {
                    text: s.into(),
                    value: t,
                }
            }).collect(),
        }
    }

    fn strings(&self) -> Vec<String> {
        self.choices.iter().map(|c| c.text.clone()).collect()
    }
}

#[derive(Debug, Clone)]
pub enum ElementHandle {
    AbsDiv(AbsDiv),
    Text(Text),
    RichText(RichText),
    Canvas(Canvas),
    BorderContainer(BorderContainer),
    Mono(Mono),
    MenuPlace(MenuPlace),
}

impl ElementHandle {
    pub fn abs_div(&self) -> Option<&AbsDiv> {
        if let &ElementHandle::AbsDiv(ref e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn text(&self) -> Option<&Text> {
        if let &ElementHandle::Text(ref e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub fn canvas(&self) -> Option<&Canvas> {
        if let &ElementHandle::Canvas(ref e) = self {
            Some(e)
        } else {
            None
        }
    }
    pub(crate) fn render(&self, grid: &mut Grid<Cell>, seq: u64, offset: Vector2<i16>, depth: i16) {
        match self {
            &ElementHandle::AbsDiv(ref e) => e.render(grid, seq, offset, depth),
            &ElementHandle::Text(ref e) => e.render(grid, seq, offset, depth),
            &ElementHandle::Canvas(ref e) => e.render(grid, seq, offset, depth),
            &ElementHandle::BorderContainer(ref e) => e.render(grid, seq, offset, depth),
            &ElementHandle::RichText(ref e) => e.render(grid, seq, offset, depth),
            &ElementHandle::Mono(ref e) => e.render(grid, seq, offset, depth),
            &ElementHandle::MenuPlace(ref e) => e.render(grid, seq, offset, depth),
        }
    }
    pub(crate) fn size(&self) -> Vector2<u16> {
        match self {
            &ElementHandle::AbsDiv(ref e) => e.size(),
            &ElementHandle::Text(ref e) => e.size(),
            &ElementHandle::Canvas(ref e) => e.size(),
            &ElementHandle::BorderContainer(ref e) => e.size(),
            &ElementHandle::RichText(ref e) => e.size(),
            &ElementHandle::Mono(ref e) => e.size(),
            &ElementHandle::MenuPlace(ref e) => e.size(),
        }
    }
    pub(crate) fn find_menu_place(&self, name: &str) -> Option<MenuPlace> {
        match self {
            &ElementHandle::MenuPlace(ref e) => {
                if e.name_matches(name) {
                    return Some(e.clone());
                }
            }
            &ElementHandle::Mono(ref e) => {
                return e.find_menu_place(name);
            }
            &ElementHandle::BorderContainer(ref e) => {
                return e.find_menu_place(name);
            }
            &ElementHandle::AbsDiv(ref e) => {
                return e.find_menu_place(name);
            }
            _ => {}
        }
        None
    }
}

impl From<AbsDiv> for ElementHandle {
    fn from(e: AbsDiv) -> Self {
        ElementHandle::AbsDiv(e)
    }
}
impl From<Text> for ElementHandle {
    fn from(e: Text) -> Self {
        ElementHandle::Text(e)
    }
}
impl From<Canvas> for ElementHandle {
    fn from(e: Canvas) -> Self {
        ElementHandle::Canvas(e)
    }
}
impl From<BorderContainer> for ElementHandle {
    fn from(e: BorderContainer) -> Self {
        ElementHandle::BorderContainer(e)
    }
}
impl From<RichText> for ElementHandle {
    fn from(e: RichText) -> Self {
        ElementHandle::RichText(e)
    }
}
impl From<Mono> for ElementHandle {
    fn from(e: Mono) -> Self {
        ElementHandle::Mono(e)
    }
}
impl From<MenuPlace> for ElementHandle {
    fn from(e: MenuPlace) -> Self {
        ElementHandle::MenuPlace(e)
    }
}
