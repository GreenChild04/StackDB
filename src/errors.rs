//! Errors used in the `stack-db` crate

use std::fmt::Display;

/// Errors used in the `stack-db` crate
#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    DBCorrupt(Box<Error>),
    ReadOnly,
    InvalidLayer,
    OutOfBounds,
    Custom(String),
}

impl std::error::Error for Error {}
impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}
