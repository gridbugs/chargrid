use term::terminfo::TermInfo;
use error::{Error, Result};

pub struct TermInfoStrings {
    pub enter_ca: Vec<u8>,
    pub exit_ca: Vec<u8>,
    pub enter_xmit: Vec<u8>,
    pub exit_xmit: Vec<u8>,
    pub show_cursor: Vec<u8>,
    pub hide_cursor: Vec<u8>,
    pub clear: Vec<u8>,
    pub reset: Vec<u8>,
    pub set_cursor: Vec<u8>,
}

impl TermInfoStrings {
    pub fn new() -> Result<Self> {

        let term_info = TermInfo::from_env()?;

        let cap = |name: &'static str| {
            term_info.strings.get(name).cloned().ok_or_else(|| {
                Error::MissingCap(name.to_string())
            })
        };

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
        })
    }
}
