use crate::{
    error::{SqResult, SqpackError},
    io::{
        dat::ContentType,
        index::{IndexFileEntry, IndexReader},
    },
    SqPath,
};
use byteorder::{ReadBytesExt, LE};
use flate2::bufread::DeflateDecoder;
use std::{
    convert::TryInto,
    fs::File,
    io::{Cursor, Error as IOError, ErrorKind, Read, Seek, SeekFrom},
    path::Path,
    vec::IntoIter as VecIntoIter,
};

/// Allows reading files within a SqPack using Rust's [`Read`][read]
/// trait. Within the SqPack, files are compressed, so there is necessarily
/// an allocated buffer.
///
/// [read]: https://doc.rust-lang.org/std/io/trait.Read.html
pub struct SqFile<R: Read + Seek> {
    /// Inner reader
    inner: R,
    /// The index entry of the file we're reading
    index_entry: IndexFileEntry,
    /// The blocks as a Vec's IntoIter that were read during reading the table
    blocks: VecIntoIter<BlockTableEntry>,
    /// The current block this file is currently reading. This contains the buffer.
    current_block: Option<ReadingBlock>,
    /// The information about the data in the dat file.
    dat_info: DatInfo,
}

impl SqFile<File> {
    /// Opens a file within the SqPack from a [`SqPath`](../../../sqpath/struct.SqPath.html).
    /// Also needs a path to the SqPack directory on the OS.
    pub fn open_sqpath<SQ, P>(sqpath: SQ, sqpack: P) -> SqResult<SqFile<File>>
    where
        SQ: AsRef<SqPath>,
        P: AsRef<Path>,
    {
        let sqpath = sqpath.as_ref();
        let sqpack = sqpack.as_ref();

        let index_hash = sqpath.sq_index_hash().ok_or(SqpackError::SqFileNotFound)?;
        let mut index_path = sqpath
            .sqpack_index_path(sqpack)
            .ok_or(SqpackError::SqFileNotFound)?;

        // Open a reader to find the right file
        let mut index_reader = IndexReader::new(File::open(index_path.as_path())?)?;
        let mut entry_opt = None;
        for file_res in index_reader.files()? {
            let file = file_res?;
            if file.path_hash == index_hash {
                entry_opt = Some(file);
                break;
            }
        }

        // Get the entry, using it set the path's dat file number
        let entry = entry_opt.ok_or(SqpackError::SqFileNotFound)?;
        let mut ext = [0x64u8, 0x61, 0x74, 0x30];
        ext[3] += entry.dat_file;
        let ext = std::str::from_utf8(&ext).map_err(|_| IOError::from(ErrorKind::InvalidData))?;
        index_path.set_extension(ext);

        // Open the file and pass it to the reader function
        let dat_file = File::open(index_path)?;
        Self::open_reader(dat_file, entry)
    }
}

impl<R: Read + Seek> SqFile<R> {
    /// Opens a file within the SqPack given a .dat reader. If the passed index
    /// entry is not found within this dat file, you will get corrupted data,
    /// or more likely just get errors on reading.
    pub fn open_reader(reader: R, index_entry: IndexFileEntry) -> SqResult<SqFile<R>> {
        let mut reader = reader;
        let dat_info = DatInfo::read_header(&mut reader, &index_entry)?;
        let blocks = read_block_table_entries(&mut reader, &index_entry, &dat_info)?.into_iter();
        Ok(SqFile {
            inner: reader,
            index_entry,
            blocks,
            current_block: None,
            dat_info,
        })
    }

    /// Reopens this SqFile reader on a new index file, without having to construct
    /// a new reader.
    pub fn reopen(self, index_entry: IndexFileEntry) -> SqResult<SqFile<R>> {
        let mut slf = self;
        slf.dat_info = DatInfo::read_header(&mut slf.inner, &index_entry)?;
        slf.current_block = None;
        slf.blocks =
            read_block_table_entries(&mut slf.inner, &index_entry, &slf.dat_info)?.into_iter();
        slf.index_entry = index_entry;
        Ok(slf)
    }

    /// Begins reading a block. Loads the block data into a buffer and
    /// determines if it needs decompression.
    fn start_block(&mut self, entry: BlockTableEntry) -> Result<ReadingBlock, IOError> {
        let block_offset = self.index_entry.data_offset + self.dat_info.header_len + entry.offset;
        self.inner.seek(SeekFrom::Start(block_offset as u64))?;

        // Read the header into a buffer
        let mut header = [0u8; 0x10];
        self.inner.read_exact(&mut header)?;
        let mut cursor = Cursor::new(header);

        // Read the header data
        let block_header_len = cursor.read_u32::<LE>()?;
        cursor.seek(SeekFrom::Current(4))?;
        let compressed_len = cursor.read_u32::<LE>()?;
        let decompressed_len = cursor.read_u32::<LE>()?;

        // According to datamining research, if the compressed_len is < 32000,
        // it is compressed. Otherwise it should be exactly 32000
        let is_compressed = compressed_len < 32000;
        if !is_compressed && compressed_len != 32000 {
            return Err(IOError::from(ErrorKind::InvalidData));
        }

        // Determine the length of the data to read from the file
        let final_length = if is_compressed {
            if (entry.block_size as u32 + block_header_len) % BLOCK_PADDING != 0 {
                compressed_len + BLOCK_PADDING
                    - ((entry.block_size as u32 - block_header_len) % BLOCK_PADDING)
            } else {
                compressed_len
            }
        } else {
            decompressed_len
        };

        // Create a buffer by creating a Vec and converting it into a boxed slice
        // This unsafe call uses the exact same strategy as std's BufReader
        let mut data = Vec::with_capacity(final_length as usize);
        let mut data = unsafe {
            data.set_len(final_length as usize);
            data.into_boxed_slice()
        };

        // Read all of the data into the buffer
        self.inner.read_exact(&mut data)?;
        let data = Cursor::new(data);
        Ok(if is_compressed {
            ReadingBlock::Compressed(CompressedReadingBlock {
                decoder: DeflateDecoder::new(data),
            })
        } else {
            ReadingBlock::Uncompressed(UncompressedReadingBlock { buffer: data })
        })
    }

    /// Retrieves the kind of content stored within this .dat file
    pub fn content_type(&self) -> ContentType { self.dat_info.content_type }

    /// Retrieves the resulting size of this file stored within the SqPack.
    /// This can be used to prepare an in-memory buffer.
    pub fn total_size(&self) -> usize { self.dat_info.uncompressed_size as usize }
}

/// The padding between the block header and the block data
const BLOCK_PADDING: u32 = 0x80;

impl<R: Read + Seek> Read for SqFile<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        // check if we're in the middle of a block
        if let Some(current_block) = self.current_block.as_mut() {
            // read from the block
            let res = current_block.read(buf);
            if let Ok(n) = res {
                if n == 0 {
                    // if there was nothing left in the current block
                    // start reading the next block
                    // and recurse to read that block
                    if let Some(next) = self.blocks.next() {
                        let reading_block = self.start_block(next)?;
                        self.current_block.replace(reading_block);
                        let r = self.read(buf)?;
                        Ok(r)
                    } else {
                        // if there was no other block, we're done
                        self.current_block = None;
                        Ok(0)
                    }
                } else {
                    Ok(n)
                }
            } else {
                Err(res.unwrap_err())
            }
        } else {
            // Not currently reading a block
            if let Some(next) = self.blocks.next() {
                // if there is a block to read, do that and recurse
                let reading_block = self.start_block(next)?;
                self.current_block.replace(reading_block);
                self.read(buf)
            } else {
                // otherwise done
                Ok(0)
            }
        }
    }
}

/// Used to determine which type of block is being read
enum ReadingBlock {
    /// A DEFLATE compressed block
    Compressed(CompressedReadingBlock),
    /// An uncompressed block
    Uncompressed(UncompressedReadingBlock),
}

/// Wraps a deflate decoder over a buffer of the block
struct CompressedReadingBlock {
    pub decoder: DeflateDecoder<Cursor<Box<[u8]>>>,
}

/// Wraps a cursor over a buffer of the block
struct UncompressedReadingBlock {
    pub buffer: Cursor<Box<[u8]>>,
}

// Dispatches read calls
impl Read for ReadingBlock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
        match self {
            ReadingBlock::Compressed(c) => c.read(buf),
            ReadingBlock::Uncompressed(u) => u.read(buf),
        }
    }
}

impl Read for CompressedReadingBlock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> { self.decoder.read(buf) }
}

impl Read for UncompressedReadingBlock {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> { self.buffer.read(buf) }
}

/// Information about the file (the data header)
struct DatInfo {
    pub header_len: u32,
    pub content_type: ContentType,
    pub uncompressed_size: u32,
    pub blocks_len: u32,
}

impl DatInfo {
    /// Reads the header from a reader given that its opened to a .dat file with the
    /// provided index entry contained.
    pub(crate) fn read_header<R>(reader: &mut R, index_entry: &IndexFileEntry) -> SqResult<DatInfo>
    where
        R: Read + Seek,
    {
        reader.seek(SeekFrom::Start(index_entry.data_offset as u64))?;
        let mut buffer = {
            let mut buf = crate::buffer(24);
            reader.read_exact(buf.as_mut())?;
            Cursor::new(buf)
        };
        let header_len = buffer.read_u32::<LE>()?;
        let content_type: ContentType = buffer.read_u32::<LE>()?.try_into()?;
        let uncompressed_size = buffer.read_u32::<LE>()?;
        buffer.seek(SeekFrom::Current(8))?;
        let blocks_len = buffer.read_u32::<LE>()?;

        Ok(DatInfo {
            header_len,
            content_type,
            uncompressed_size,
            blocks_len,
        })
    }
}

/// Data about a block of a SqFile
struct BlockTableEntry {
    /// The offset relative to the end of the header of the SqFile
    offset: u32,
    /// The compressed size of the block
    block_size: u16,
}

/// Take a reader and an index entry and the SqFile's header info and read the block
/// table to produce a vector over the block information.
fn read_block_table_entries<R>(
    reader: &mut R,
    index_entry: &IndexFileEntry,
    dat_info: &DatInfo,
) -> SqResult<Vec<BlockTableEntry>>
where
    R: Read + Seek,
{
    reader.seek(SeekFrom::Start(index_entry.data_offset as u64 + 24))?;

    let mut blocks = Vec::with_capacity(dat_info.blocks_len as usize);

    assert_eq!(
        dat_info.content_type,
        ContentType::Binary,
        "The content type is unsupported."
    );
    // TODO implement support for other content types

    let mut buffer = {
        let buf_size = 8 * dat_info.blocks_len as usize;
        // create a buffer for all the blocks and read in all at once
        let mut buffer = crate::buffer(buf_size);
        reader.read_exact(buffer.as_mut())?;
        Cursor::new(buffer)
    };

    // read the blocks
    for _ in 0..dat_info.blocks_len {
        blocks.push(BlockTableEntry {
            offset: buffer.read_u32::<LE>()?,
            block_size: buffer.read_u16::<LE>()?,
        });
        buffer.seek(SeekFrom::Current(2))?;
    }

    Ok(blocks)
}
