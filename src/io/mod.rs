/// Types and functions relating to opening and reading specific files within the Sqpack
pub mod sqfile;

/// Types and functions relating to index files.
pub mod index;

/// Module for errors specific to SqPack IO
mod error;

pub use self::error::{SqResult, SqpackError};
