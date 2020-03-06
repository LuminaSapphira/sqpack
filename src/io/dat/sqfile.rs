use std::io::{Seek, Read, SeekFrom, BufReader, Error as IOError, Cursor, ErrorKind};
use io::index::IndexFileEntry;
use error::SqResult;
use io::dat::ContentType;
use byteorder::{LE, ReadBytesExt};
use std::convert::TryInto;
use flate2::bufread::DeflateDecoder;
use std::cmp::min;
use std::vec::IntoIter as VecIntoIter;
use std::fs::File;
use SqPath;
use std::path::Path;

pub struct SqFile<R: Read + Seek> {
    inner: R,
    index_entry: IndexFileEntry,
    blocks: VecIntoIter<BlockTableEntry>,
    current_block: Option<ReadingBlock>,
    dat_info: DatInfo,
}

impl SqFile<File> {
    pub fn open_sqpack<SQ, P>(sqpath: SQ, sqpack: P) -> SqResult<SqFile<File>>
        where SQ: AsRef<SqPath>, P: AsRef<Path>
    {
        let sqpath = sqpath.as_ref();
        let sqpack = sqpack.as_ref();

        unimplemented!()
    }
}

impl<R: Read + Seek> SqFile<R> {

    pub fn open_reader(reader: R, index_entry: IndexFileEntry) -> SqResult<SqFile<R>> {
        let mut reader = reader;
        let dat_info = DatInfo::read_header(&mut reader, &index_entry)?;
        let blocks = read_block_table_entries(&mut reader, &index_entry, &dat_info)?.into_iter();
        Ok(SqFile { inner: reader, index_entry, blocks, current_block: None, dat_info})
    }

    fn start_block(&mut self, entry: BlockTableEntry) -> Result<ReadingBlock, IOError> {
        let block_offset = self.index_entry.data_offset + self.dat_info.header_len + entry.offset;
        self.inner.seek(SeekFrom::Start(block_offset as u64))?;
        let block_header_len = self.inner.read_u32::<LE>()?;
        self.inner.seek(SeekFrom::Current(4))?;
        let compressed_len = self.inner.read_u32::<LE>()?;
        let decompressed_len = self.inner.read_u32::<LE>()?;
        let is_compressed = compressed_len < 32000;
        let final_length =
            if is_compressed {
                if (entry.block_size as u32 + block_header_len) % BLOCK_PADDING != 0 {
                    compressed_len + BLOCK_PADDING - ((entry.block_size as u32 - block_header_len) % BLOCK_PADDING)
                } else { compressed_len }
            } else {
                decompressed_len
            };

        let mut data = Vec::with_capacity(final_length as usize);
        let mut data= unsafe {
            data.set_len(final_length as usize);
            data.into_boxed_slice()
        };
        self.inner.read_exact(&mut data);
        let data = Cursor::new(data);
        Ok(if is_compressed {
            ReadingBlock::Compressed(CompressedReadingBlock { decoder: DeflateDecoder::new(data ) })
        } else {
            ReadingBlock::Uncompressed(UncompressedReadingBlock { buffer: data })
        })
    }

}

const BLOCK_PADDING: u32 = 0x80;

impl<R: Read + Seek> Read for SqFile<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        if let Some(current_block) = self.current_block.as_mut() {
            let res = current_block.read(buf);
            if let Ok(n) = res {
                if n == 0 {
                    if let Some(next) = self.blocks.next() {
                        let reading_block = self.start_block(next)?;
                        self.current_block.replace(reading_block);
                        let r = self.read(&mut buf[n..])?;
                        Ok(n + r)
                    }
                    else {
                        Ok(0)
                    }
                } else {
                    Ok(n)
                }
            } else {
                Err(res.unwrap_err())
            }
        } else {
            if let Some(next) = self.blocks.next() {
                let reading_block = self.start_block(next)?;
                self.current_block.replace(reading_block);
                self.read(buf)
            } else {
                Ok(0)
            }
        }
    }
}

enum ReadingBlock {
    Compressed(CompressedReadingBlock),
    Uncompressed(UncompressedReadingBlock)
}

struct CompressedReadingBlock {
    pub decoder: DeflateDecoder<Cursor<Box<[u8]>>>,
}

struct UncompressedReadingBlock {
    pub buffer: Cursor<Box<[u8]>>,
}

impl Read for ReadingBlock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        match self {
            ReadingBlock::Compressed(c) => c.read(buf),
            ReadingBlock::Uncompressed(u) => u.read(buf)
        }
    }
}

impl Read for CompressedReadingBlock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        self.decoder.read(buf)
    }
}

impl Read for UncompressedReadingBlock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        self.buffer.read(buf)
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
