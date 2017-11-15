use std::io::Error as IoError;
use term::Error as TermError;
use term::terminfo::parm::Error as ParamError;

#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    TermError(TermError),
    MissingCap(String),
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
