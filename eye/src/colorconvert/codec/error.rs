use std::{error, fmt, io, result};

//
// Modeled after std::io::error: https://doc.rust-lang.org/src/std/io/error.rs.html.
//

/// Specialized result type for this crate.
pub type Result<T> = result::Result<T, Error>;

/// Error type for all fallible operations in this crate.
#[derive(Debug)]
pub struct Error {
    repr: Repr,
}

enum Repr {
    Simple(ErrorKind),
    Custom(Box<Custom>),
}

impl fmt::Debug for Repr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Repr::Simple(kind) => fmt.debug_tuple("Kind").field(&kind).finish(),
            Repr::Custom(ref c) => fmt::Debug::fmt(&c, fmt),
        }
    }
}

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn error::Error + Send + Sync>,
}

#[derive(Debug)]
pub enum ErrorKind {
    /// Invalid buffer.
    InvalidBuffer,
    /// Invalid parameter.
    InvalidParam,
    /// Format not supported.
    UnsupportedFormat,
    /// Any other error not part of this list.
    Other,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<Box<dyn error::Error + Send + Sync>>,
    {
        Error {
            repr: Repr::Custom(Box::new(Custom {
                kind,
                error: error.into(),
            })),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            repr: Repr::Simple(kind),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error {
            repr: Repr::Custom(Box::new(Custom {
                kind: ErrorKind::Other,
                error: error.into(),
            })),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.repr {
            Repr::Simple(kind) => write!(fmt, "{}", kind),
            Repr::Custom(ref c) => c.error.fmt(fmt),
        }
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}
