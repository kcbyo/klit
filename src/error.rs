use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    BadUrl(url::ParseError),
    Io(io::Error),
    MissingDomain(String),
    Reqwest(reqwest::Error),
    UnknownDomain(String),
}

impl From<url::ParseError> for Error {
    fn from(v: url::ParseError) -> Self {
        Self::BadUrl(v)
    }
}

impl From<io::Error> for Error {
    fn from(v: io::Error) -> Self {
        Error::Io(v)
    }
}

impl From<reqwest::Error> for Error {
    fn from(v: reqwest::Error) -> Self {
        Self::Reqwest(v)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadUrl(e) => e.fmt(f),
            Error::Io(e) => e.fmt(f),
            Error::MissingDomain(value) => write!(f, "missing domain: {}", value),
            Error::UnknownDomain(value) => write!(f, "unknown domain: {}", value),
            Error::Reqwest(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}
