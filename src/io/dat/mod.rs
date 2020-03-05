use std::convert::TryFrom;
use error::SqpackError;

pub mod sqfile;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum ContentType {
    Empty,
    Binary,
    Model,
    Texture,
}

impl TryFrom<u32> for ContentType {
    type Error = SqpackError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(ContentType::Empty),
            2 => Ok(ContentType::Binary),
            3 => Ok(ContentType::Model),
            4 => Ok(ContentType::Texture),
            unk => Err(SqpackError::UnknownContentType(unk))
        }
    }
}
