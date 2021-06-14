use std::{error, fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    BadUrl(url::ParseError),
    Io(io::Error),
    MissingDomain(String),
    UnknownDomain(String),
    Ureq(Box<ureq::Error>),
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

impl From<ureq::Error> for Error {
    fn from(v: ureq::Error) -> Self {
        Self::Ureq(Box::new(v))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BadUrl(e) => e.fmt(f),
            Error::Io(e) => e.fmt(f),
            Error::MissingDomain(value) => write!(f, "missing domain: {}", value),
            Error::UnknownDomain(value) => write!(f, "unknown domain: {}", value),
            Error::Ureq(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

// #[derive(Debug)]
// pub enum Error {
//     BadAddress(String),
//     BadId(String),
//     Incomplete,
//     Io(io::Error),
//     MissingTitle,
//     Ureq(Box<ureq::Error>),
// }

// impl From<ureq::Error> for Error {
//     fn from(v: ureq::Error) -> Self {
//         Self::Ureq(Box::new(v))
//     }
// }

// impl From<io::Error> for Error {
//     fn from(v: io::Error) -> Self {
//         Self::Io(v)
//     }
// }

// impl Display for Error {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Error::BadAddress(url) => write!(f, "bad url: {}", url),
//             Error::BadId(id) => write!(f, "bad story id: {}", id),
//             Error::Incomplete => f.write_str("incomplete file"),
//             Error::Io(e) => e.fmt(f),
//             Error::Ureq(e) => e.fmt(f),
//             Error::MissingTitle => f.write_str("invalid response: missing title"),
//         }
//     }
// }

// impl error::Error for Error {}
