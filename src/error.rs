use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::io::Error as IOError;

/// Errors specific to Sqpack I/O
pub enum SqpackError {
    SqFileNotFound,
    IO(IOError),
    ReaderIsNotSqPack,
    ReaderIsNotIndex,
    UnknownContentType(u32),
}

pub type SqResult<T> = Result<T, SqpackError>;

impl Error for SqpackError {}

impl Display for SqpackError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{:?}", self)
    }
}

impl Debug for SqpackError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::SqFileNotFound => write!(f, "SqPath not found in index!"),
            Self::IO(err) => write!(f, "Underlying IO Error ({:?})", err),
            Self::ReaderIsNotSqPack => write!(f, "The underlying reader is not SqPack data"),
            Self::ReaderIsNotIndex => write!(f, "The underlying reader is not SqPack Index data"),
            Self::UnknownContentType(unk) => write!(f, "Unknown content type found while reading .dat: {}", unk)
        }
    }
}

impl From<IOError> for SqpackError {
    fn from(err: IOError) -> Self {
        SqpackError::IO(err)
    }
}
