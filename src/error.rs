use std::{error, fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    BadAddress(String),
    BadId(String),
    Incomplete,
    Io(io::Error),
    MissingTitle,
    Ureq(Box<ureq::Error>),
}

impl From<ureq::Error> for Error {
    fn from(v: ureq::Error) -> Self {
        Self::Ureq(Box::new(v))
    }
}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Self::Io(v)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadAddress(url) => write!(f, "bad url: {}", url),
            Error::BadId(id) => write!(f, "bad story id: {}", id),
            Error::Incomplete => f.write_str("incomplete file"),
            Error::Io(e) => e.fmt(f),
            Error::Ureq(e) => e.fmt(f),
            Error::MissingTitle => f.write_str("invalid response: missing title"),
        }
    }
}

impl error::Error for Error {}
