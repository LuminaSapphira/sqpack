use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};

/// Errors specific to Sqpack I/O
pub enum SqpackError {
    /// A file was not found within an index or .dat file,
    /// or a SqPath failed to resolve.
    SqFileNotFound,
    /// An IO Error occurred
    IO(IOError),
    /// The IndexReader was not initialized over an index file
    IndexReaderIsNotIndex,
    /// The content type read from a .dat file was unknown.
    /// This usually means a SqFile was attempted to be initialized
    /// over the incorrect .dat file reader. If the value contained
    /// is 1, 3, or 4 however, it is valid, but we do not support those
    /// ContentTypes yet.
    UnknownContentType(u32),
}

/// Simple result wrapper that uses SqpackError for errors
pub type SqResult<T> = Result<T, SqpackError>;

impl Error for SqpackError {}

impl Display for SqpackError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult { write!(f, "{:?}", self) }
}

impl Debug for SqpackError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::SqFileNotFound => write!(f, "SqPath not found in index!"),
            Self::IO(err) => write!(f, "Underlying IO Error ({:?})", err),
            Self::IndexReaderIsNotIndex => {
                write!(f, "The underlying reader is not SqPack Index data")
            }
            Self::UnknownContentType(unk) => {
                write!(f, "Unknown content type found while reading .dat: {}", unk)
            }
        }
    }
}

impl From<IOError> for SqpackError {
    fn from(err: IOError) -> Self { SqpackError::IO(err) }
}
