use super::byte_prefix_tree::BytePrefixTree;
use ansi_colour::{AllColours, Colour};
use error::{Error, Result};
use prototty_input::{Input, MouseButton, ScrollDirection};
use term::terminfo::parm::{self, Param, Variables};
use term::terminfo::TermInfo;

// XXX this might not be portable
const ESCAPE: &'static [u8] = &[27];
const ENABLE_MOUSE_REPORTING: &'static str = "[?1003h";
const DISABLE_MOUSE_REPORTING: &'static str = "[?1003l";

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
    pub enter_ca: String,
    pub exit_ca: String,
    pub enter_xmit: String,
    pub exit_xmit: String,
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
        for colour in AllColours {
            let code = colour.code() as i32;
            let params = &[Param::Number(code)];
            fg_colours.push(::std::str::from_utf8(&parm::expand(setfg.as_bytes(), params, &mut vars)?)?.to_string());
            bg_colours.push(::std::str::from_utf8(&parm::expand(setbg.as_bytes(), params, &mut vars)?)?.to_string());
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
            let mut bytes = ESCAPE.iter().cloned().collect::<Vec<_>>();
            for byte in seq.bytes() {
                bytes.push(byte);
            }

            (bytes, input)
        };

        let inputs_to_escape = [
            escseq("kf1", Input::Function(1))?,
            escseq("kf2", Input::Function(2))?,
            escseq("kf3", Input::Function(3))?,
            escseq("kf4", Input::Function(4))?,
            escseq("kf5", Input::Function(5))?,
            escseq("kf6", Input::Function(6))?,
            escseq("kf7", Input::Function(7))?,
            escseq("kf8", Input::Function(8))?,
            escseq("kf9", Input::Function(9))?,
            escseq("kf10", Input::Function(10))?,
            escseq("kf11", Input::Function(11))?,
            escseq("kf12", Input::Function(12))?,
            escseq("kcuu1", Input::Up)?,
            escseq("kcud1", Input::Down)?,
            escseq("kcuf1", Input::Right)?,
            escseq("kcub1", Input::Left)?,
            escseq("kpp", Input::PageUp)?,
            escseq("knp", Input::PageDown)?,
            escseq("khome", Input::Home)?,
            escseq("kend", Input::End)?,
            escseq("kdch1", Input::Delete)?,
            raw_escseq("[MC", TerminalInput::MousePrefix(MousePrefix::Move(None))),
            raw_escseq("[M ", TerminalInput::MousePrefix(MousePrefix::Press(MouseButton::Left))),
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
            enter_ca: cap("smcup")?,
            exit_ca: cap("rmcup")?,
            enter_xmit: cap("smkx")?,
            exit_xmit: cap("rmkx")?,
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

    pub fn fg_colour(&self, colour: Colour) -> &str {
        self.fg_colours[colour.code() as usize].as_str()
    }

    pub fn bg_colour(&self, colour: Colour) -> &str {
        self.bg_colours[colour.code() as usize].as_str()
    }
}
