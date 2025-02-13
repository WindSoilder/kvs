use bincode::Error as BincodeError;
use serde_json::Error as JsonError;
use sled::Error as SledError;
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::io;
use std::result::Result as StdResult;
use std::string::FromUtf8Error;

pub type Result<T> = StdResult<T, KvsError>;

#[derive(Debug)]
pub enum Repr {
    IOError(io::Error),
    BinCodeError(BincodeError),
    SledError(SledError),
    JsonError(JsonError),
    FromUtf8Error(FromUtf8Error),
    CommandError(String),
    StorageEngineError(String),
}

#[derive(Debug)]
pub struct KvsError {
    repr: Repr,
}

impl Error for KvsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.repr {
            Repr::IOError(e) => e.source(),
            Repr::BinCodeError(e) => e.source(),
            Repr::JsonError(e) => e.source(),
            Repr::SledError(e) => e.source(),
            Repr::FromUtf8Error(e) => e.source(),
            Repr::CommandError(_) => None,
            Repr::StorageEngineError(_) => None,
        }
    }
}

impl From<FromUtf8Error> for KvsError {
    fn from(error: FromUtf8Error) -> Self {
        KvsError {
            repr: Repr::FromUtf8Error(error),
        }
    }
}

impl From<SledError> for KvsError {
    fn from(error: SledError) -> Self {
        KvsError {
            repr: Repr::SledError(error),
        }
    }
}

impl From<io::Error> for KvsError {
    fn from(error: io::Error) -> Self {
        KvsError {
            repr: Repr::IOError(error),
        }
    }
}

impl From<BincodeError> for KvsError {
    fn from(error: BincodeError) -> Self {
        KvsError {
            repr: Repr::BinCodeError(error),
        }
    }
}

impl From<JsonError> for KvsError {
    fn from(error: JsonError) -> Self {
        KvsError {
            repr: Repr::JsonError(error),
        }
    }
}

impl Display for KvsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.repr)
    }
}

impl KvsError {
    pub fn from_string(msg: &str) -> KvsError {
        KvsError {
            repr: Repr::CommandError(msg.to_owned()),
        }
    }

    pub fn from_unsupported_engine(msg: &str) -> KvsError {
        KvsError {
            repr: Repr::StorageEngineError(String::from(msg)),
        }
    }

    pub fn repr(&self) -> &Repr {
        &self.repr
    }
}
