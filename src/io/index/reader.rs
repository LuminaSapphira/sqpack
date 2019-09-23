use std::io::{Read, Seek, SeekFrom};
use std::fs::File;
use SqResult;
use crate::byteorder::{ReadBytesExt, LE};

/// Reads the header length from a `Read` mutable reference. `file` should be an
/// opened .win32.index file from the SqPack, or other comparable read instance.
///
/// # Returns
/// The header length wrapped in a result. The file reference is mutated in that its cursor
/// position may be different.
pub fn header_length<F: Read + Seek>(file: &mut F) -> SqResult<u32> {
    file.seek(SeekFrom::Start(0x0c))?;
    let len = file.read_u32::<LE>()?;
    Ok(len)
}


