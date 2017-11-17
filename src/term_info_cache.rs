use term::terminfo::TermInfo;
use term::terminfo::parm::{self, Param, Variables};
use terminal_colour::{Colour, AllColours};
use error::{Error, Result};

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
    pub fg_colours: Vec<Vec<u8>>,
    pub bg_colours: Vec<Vec<u8>>,
    pub vars: Variables,
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
            fg_colours,
            bg_colours,
            vars,
        })
    }

    pub fn fg_colour(&self, colour: Colour) -> &[u8] {
        &self.fg_colours[colour.code() as usize]
    }

    pub fn bg_colour(&self, colour: Colour) -> &[u8] {
        &self.bg_colours[colour.code() as usize]
    }
}
