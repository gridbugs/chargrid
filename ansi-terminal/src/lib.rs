#[cfg(unix)]
mod context;
#[cfg(unix)]
mod error;
#[cfg(unix)]
mod terminal;

#[cfg(unix)]
mod public {
    pub use super::context::*;
    pub use super::error::*;
    pub use super::terminal::col_encode;
    pub use super::terminal::ColEncode;
    pub use chargrid_input;
}

#[cfg(unix)]
pub use public::*;

#[cfg(not(unix))]
const WARNING: &str = "compiling to empty crate on non-unix system";
