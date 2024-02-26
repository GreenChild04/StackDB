#[derive(Debug)]
pub enum Error {
    IOError(std::io::Error),
    DBCorrupt(Box<Error>),
    ReadOnly,
    InvalidLayer,
    OutOfBounds,
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}
