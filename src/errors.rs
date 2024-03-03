//! Errors used in the `stack-db` crate

use std::fmt::Display;

/// Errors used in the `stack-db` crate
#[derive(Debug)]
pub enum Error {
    /// An io error
    IOError(std::io::Error),
    /// An error that corrupts the database
    DBCorrupt(Box<Error>),
    /// If you try to write on a read-only layer
    ReadOnly,
    /// When the layer meta-data is invalid
    InvalidLayer,
    /// When there is an out of bounds read
    OutOfBounds,
    /// A custom error
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
