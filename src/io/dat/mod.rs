use std::convert::TryFrom;
use error::SqpackError;

mod sqfile;
pub use self::sqfile::SqFile;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
/// The type of content stored within a file stored within a .dat file of a SqPack
pub enum ContentType {
    /// An empty placeholder file **(currently unsupported)**
    Empty,
    /// Binary data file (typically anything not a Model or Texture)
    Binary,
    /// Model data file **(currently unsupported)**
    Model,
    /// Texture data file **(currently unsupported)**
    Texture,
}

impl TryFrom<u32> for ContentType {
    type Error = SqpackError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            // 1 => Ok(ContentType::Empty),
            2 => Ok(ContentType::Binary),
            // 3 => Ok(ContentType::Model),
            // 4 => Ok(ContentType::Texture),
            unk => Err(SqpackError::UnknownContentType(unk))
        }
    }
}
