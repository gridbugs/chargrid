use std::io::Error as IoError;
use std::str::Utf8Error;
use term::terminfo::parm::Error as ParamError;
use term::Error as TermError;

#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    MissingCap(String),
    UnrecognizedEscapeSequence(Vec<u8>),
    Utf8Error(Utf8Error),
    NoSuchMenuPlace(String),
    TermError(TermError),
    ParamError(ParamError),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl Error {
    pub fn last_os_error() -> Self {
        Error::IoError(IoError::last_os_error())
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Self {
        Error::IoError(e)
    }
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Error::Utf8Error(e)
    }
}

impl From<TermError> for Error {
    fn from(e: TermError) -> Self {
        Error::TermError(e)
    }
}

impl From<ParamError> for Error {
    fn from(e: ParamError) -> Self {
        Error::ParamError(e)
    }
}
