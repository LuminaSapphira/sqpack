
use std::error::Error;
use std::fmt::{Debug, Result as FmtResult, Formatter, Display};

pub mod sqfile;

/// Errors specific to Sqpack I/O
pub enum SqpackError {
    SqFileNotFound,
}

impl Error for SqpackError {}

impl Display for SqpackError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult { write!(f, "{:?}", self) }
}

impl Debug for SqpackError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::SqFileNotFound => write!(f, "SqFile not found in index!"),
        }
    }
}
