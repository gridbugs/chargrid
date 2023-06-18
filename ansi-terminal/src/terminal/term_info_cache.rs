use super::byte_prefix_tree::BytePrefixTree;
use crate::error::{Error, Result};
use chargrid_input::{Input, Key, MouseButton, ScrollDirection};
use term::terminfo::parm::{self, Param, Variables};
use term::terminfo::TermInfo;

// XXX this might not be portable
const ESCAPE: &[u8] = &[27];
const ENABLE_MOUSE_REPORTING: &str = "[?1003h";
const DISABLE_MOUSE_REPORTING: &str = "[?1003l";

#[derive(Debug, Clone, Copy)]
pub enum MousePrefix {
    Press(MouseButton),
    // ansi terminals don't report which button was released
    Release,
    // ansi terminals only report the last button pressed when reporting a mouse drag
    Move(Option<MouseButton>),
    Scroll(ScrollDirection),
}

#[derive(Debug, Clone, Copy)]
pub enum TerminalInput {
    Char(char),
    Literal(Input),
    MousePrefix(MousePrefix),
}

pub struct TermInfoCache {
    pub enter_ca: Option<String>,
    pub exit_ca: Option<String>,
    pub enter_xmit: Option<String>,
    pub exit_xmit: Option<String>,
    pub show_cursor: String,
    pub hide_cursor: String,
    pub clear: String,
    pub reset: String,
    pub set_cursor: String,
    pub bold: String,
    pub underline: String,
    pub no_underline: String,
    pub enable_mouse_reporting: String,
    pub disable_mouse_reporting: String,
    pub fg_colours: Vec<String>,
    pub bg_colours: Vec<String>,
    pub vars: Variables,
    pub escape_sequence_prefix_tree: BytePrefixTree<TerminalInput>,
}

fn raw_cap(seq: &str) -> Result<String> {
    let escape_str = ::std::str::from_utf8(ESCAPE).map_err(Error::Utf8Error)?;
    let mut string = escape_str.to_string();
    string.push_str(seq);
    Ok(string)
}

impl TermInfoCache {
    pub fn new() -> Result<Self> {
        let term_info = TermInfo::from_env()?;
        let cap = |name: &'static str| {
            term_info
                .strings
                .get(name)
                .ok_or_else(|| Error::MissingCap(name.to_string()))
                .and_then(|bytes| {
                    ::std::str::from_utf8(bytes)
                        .map(|s| s.to_string())
                        .map_err(Error::Utf8Error)
                })
        };
        let mut vars = Variables::new();
        let setfg = cap("setaf")?;
        let setbg = cap("setab")?;
        let mut fg_colours = Vec::with_capacity(256);
        let mut bg_colours = Vec::with_capacity(256);
        for code in 0..=255 {
            let params = &[Param::Number(code)];
            fg_colours.push(
                ::std::str::from_utf8(&parm::expand(setfg.as_bytes(), params, &mut vars)?)?
                    .to_string(),
            );
            bg_colours.push(
                ::std::str::from_utf8(&parm::expand(setbg.as_bytes(), params, &mut vars)?)?
                    .to_string(),
            );
        }
        let escseq = |name: &'static str, input: Input| {
            term_info
                .strings
                .get(name)
                .cloned()
                .ok_or_else(|| Error::MissingCap(name.to_string()))
                .map(|seq| (seq, TerminalInput::Literal(input)))
        };
        let raw_escseq = |seq: &'static str, input: TerminalInput| {
            let mut bytes = ESCAPE.to_vec();
            for byte in seq.bytes() {
                bytes.push(byte);
            }
            (bytes, input)
        };
        let inputs_to_escape = [
            escseq("kf1", Input::key_press(Key::Function(1)))?,
            escseq("kf2", Input::key_press(Key::Function(2)))?,
            escseq("kf3", Input::key_press(Key::Function(3)))?,
            escseq("kf4", Input::key_press(Key::Function(4)))?,
            escseq("kf5", Input::key_press(Key::Function(5)))?,
            escseq("kf6", Input::key_press(Key::Function(6)))?,
            escseq("kf7", Input::key_press(Key::Function(7)))?,
            escseq("kf8", Input::key_press(Key::Function(8)))?,
            escseq("kf9", Input::key_press(Key::Function(9)))?,
            escseq("kf10", Input::key_press(Key::Function(10)))?,
            escseq("kf11", Input::key_press(Key::Function(11)))?,
            escseq("kf12", Input::key_press(Key::Function(12)))?,
            escseq("kcuu1", Input::key_press(Key::Up))?,
            escseq("kcud1", Input::key_press(Key::Down))?,
            escseq("kcuf1", Input::key_press(Key::Right))?,
            escseq("kcub1", Input::key_press(Key::Left))?,
            escseq("kpp", Input::key_press(Key::PageUp))?,
            escseq("knp", Input::key_press(Key::PageDown))?,
            escseq("khome", Input::key_press(Key::Home))?,
            escseq("kend", Input::key_press(Key::End))?,
            escseq("kdch1", Input::key_press(Key::Delete))?,
            raw_escseq("[MC", TerminalInput::MousePrefix(MousePrefix::Move(None))),
            raw_escseq(
                "[M ",
                TerminalInput::MousePrefix(MousePrefix::Press(MouseButton::Left)),
            ),
            raw_escseq(
                "[M!",
                TerminalInput::MousePrefix(MousePrefix::Press(MouseButton::Middle)),
            ),
            raw_escseq(
                "[M\"",
                TerminalInput::MousePrefix(MousePrefix::Press(MouseButton::Right)),
            ),
            raw_escseq("[M#", TerminalInput::MousePrefix(MousePrefix::Release)),
            raw_escseq(
                "[M@",
                TerminalInput::MousePrefix(MousePrefix::Move(Some(MouseButton::Left))),
            ),
            raw_escseq(
                "[MA",
                TerminalInput::MousePrefix(MousePrefix::Move(Some(MouseButton::Middle))),
            ),
            raw_escseq(
                "[MB",
                TerminalInput::MousePrefix(MousePrefix::Move(Some(MouseButton::Right))),
            ),
            raw_escseq(
                "[M`",
                TerminalInput::MousePrefix(MousePrefix::Scroll(ScrollDirection::Up)),
            ),
            raw_escseq(
                "[Ma",
                TerminalInput::MousePrefix(MousePrefix::Scroll(ScrollDirection::Down)),
            ),
            raw_escseq(
                "[Mb",
                TerminalInput::MousePrefix(MousePrefix::Scroll(ScrollDirection::Left)),
            ),
            raw_escseq(
                "[Mc",
                TerminalInput::MousePrefix(MousePrefix::Scroll(ScrollDirection::Right)),
            ),
        ];
        let mut escape_sequence_prefix_tree = BytePrefixTree::new();
        for &(ref seq, input) in inputs_to_escape.iter() {
            escape_sequence_prefix_tree.insert(seq, input);
        }
        Ok(Self {
            enter_ca: cap("smcup").ok(),
            exit_ca: cap("rmcup").ok(),
            enter_xmit: cap("smkx").ok(),
            exit_xmit: cap("rmkx").ok(),
            show_cursor: cap("cnorm")?,
            hide_cursor: cap("civis")?,
            reset: cap("sgr0")?,
            clear: cap("clear")?,
            set_cursor: cap("cup")?,
            bold: cap("bold")?,
            underline: cap("smul")?,
            no_underline: cap("rmul")?,
            enable_mouse_reporting: raw_cap(ENABLE_MOUSE_REPORTING)?,
            disable_mouse_reporting: raw_cap(DISABLE_MOUSE_REPORTING)?,
            fg_colours,
            bg_colours,
            vars,
            escape_sequence_prefix_tree,
        })
    }

    pub fn fg_colour(&self, colour: u8) -> &str {
        self.fg_colours[colour as usize].as_str()
    }

    pub fn bg_colour(&self, colour: u8) -> &str {
        self.bg_colours[colour as usize].as_str()
    }
}
