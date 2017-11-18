use term::terminfo::TermInfo;
use term::terminfo::parm::{self, Param, Variables};
use terminal_colour::{Colour, AllColours};
use error::{Error, Result};
use input::Input;
use byte_prefix_tree::BytePrefixTree;

pub struct TermInfoCache {
    pub enter_ca: Vec<u8>,
    pub exit_ca: Vec<u8>,
    pub enter_xmit: Vec<u8>,
    pub exit_xmit: Vec<u8>,
    pub show_cursor: Vec<u8>,
    pub hide_cursor: Vec<u8>,
    pub clear: Vec<u8>,
    pub reset: Vec<u8>,
    pub set_cursor: Vec<u8>,
    pub bold: Vec<u8>,
    pub underline: Vec<u8>,
    pub fg_colours: Vec<Vec<u8>>,
    pub bg_colours: Vec<Vec<u8>>,
    pub vars: Variables,
    pub escape_sequence_prefix_tree: BytePrefixTree<Input>,
}

impl TermInfoCache {
    pub fn new() -> Result<Self> {

        let term_info = TermInfo::from_env()?;

        let cap = |name: &'static str| {
            term_info.strings.get(name).cloned().ok_or_else(|| {
                Error::MissingCap(name.to_string())
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
            fg_colours.push(parm::expand(&setfg, params, &mut vars)?);
            bg_colours.push(parm::expand(&setbg, params, &mut vars)?);
        }

        let escseq = |name: &'static str, input: Input| {
            term_info.strings.get(name).cloned()
                .ok_or_else(|| {
                    Error::MissingCap(name.to_string())
                }).map(|seq| (seq, input))
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
            fg_colours,
            bg_colours,
            vars,
            escape_sequence_prefix_tree,
        })
    }

    pub fn fg_colour(&self, colour: Colour) -> &[u8] {
        &self.fg_colours[colour.code() as usize]
    }

    pub fn bg_colour(&self, colour: Colour) -> &[u8] {
        &self.bg_colours[colour.code() as usize]
    }
}
