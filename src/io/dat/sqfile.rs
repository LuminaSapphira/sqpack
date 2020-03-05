use std::io::{Seek, Read, SeekFrom};
use io::index::IndexFileEntry;
use error::SqResult;
use io::dat::ContentType;
use byteorder::{LE, ReadBytesExt};
use std::fs::read;
use std::convert::TryInto;

pub struct SqFile<R: Read + Seek> {
    inner: R,
    index_entry: IndexFileEntry,
}

impl<R: Read + Seek> SqFile<R> {

    pub fn open_reader(reader: R, index_entry: IndexFileEntry) -> SqResult<SqFile<R>> {
        let mut reader = reader;
        let dat_info = DatInfo::read_header(&mut reader, &index_entry)?;
        let blocks = read_block_table_entries(&mut reader, &index_entry, &dat_info)?;

        unimplemented!()

    }

}

struct DatInfo {
    pub header_len: u32,
    pub content_type: ContentType,
    pub uncompressed_size: u32,
    pub block_buffer_size: u32,
    pub blocks_len: u32
}

impl DatInfo {
    pub fn read_header<R>(reader: &mut R, index_entry: &IndexFileEntry) -> SqResult<DatInfo>
        where R: Read + Seek
    {
        reader.seek(SeekFrom::Start(index_entry.data_offset as u64))?;

        let header_len = reader.read_u32::<LE>()?;
        let content_type: ContentType = reader.read_u32::<LE>()?.try_into()?;
        let uncompressed_size = reader.read_u32::<LE>()?;
        reader.seek(SeekFrom::Current(4))?;
        let block_buffer_size = reader.read_u32::<LE>()?;
        let blocks_len = reader.read_u32::<LE>()?;

        Ok(DatInfo{
            header_len,
            content_type,
            uncompressed_size,
            block_buffer_size,
            blocks_len,
        })
    }
}

struct BlockTableEntry {
    offset: u32,
    block_size: u16,
    decompressed_size: u16,
}

fn read_block_table_entries<R>(reader: &mut R, index_entry: &IndexFileEntry, dat_info: &DatInfo) -> SqResult<Vec<BlockTableEntry>>
    where R: Read + Seek
{
    reader.seek(SeekFrom::Start(index_entry.data_offset as u64 + 24))?;

    let mut blocks = Vec::with_capacity(dat_info.blocks_len as usize);

    for _ in 0..dat_info.blocks_len {
        blocks.push(BlockTableEntry {
            offset: reader.read_u32::<LE>()?,
            block_size: reader.read_u16::<LE>()?,
            decompressed_size: reader.read_u16::<LE>()?,
        })
    }

    Ok(blocks)
}
