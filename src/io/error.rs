use std::error::Error;
use std::fmt::{Debug, Result as FmtResult, Formatter, Display};
use std::io::Error as IOError;

/// Errors specific to Sqpack I/O
pub enum SqpackError {
    SqFileNotFound,
    IO(IOError),
    ReaderIsNotSqPack,
    ReaderIsNotIndex,
}

pub type SqResult<T> = Result<T, SqpackError>;

impl Error for SqpackError {}

impl Display for SqpackError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult { write!(f, "{:?}", self) }
}

impl Debug for SqpackError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::SqFileNotFound => write!(f, "SqFile not found in index!"),
            Self::IO(err) => write!(f, "Underlying IO Error ({:?})", err),
            Self::ReaderIsNotSqPack => write!(f, "The underlying reader is not SqPack data"),
            Self::ReaderIsNotIndex => write!(f, "The underlying reader is not SqPack Index data"),
        }
    }
}

impl From<IOError> for SqpackError {
    fn from(err: IOError) -> Self {
        SqpackError::IO(err)
    }
}
