use std::io::{Read, Seek, SeekFrom, BufReader};
use ::{SqResult, SqpackError};
use crate::byteorder::{ReadBytesExt, LE};
use io::index::IndexFileEntry;

/// A buffered reader that reads index files from a wrapped `Read` instance
pub struct IndexReader<R>
    where R: Read + Seek {
    inner: R,
}

/// An iterator struct over the files present in the passed IndexReader
pub struct IndexFiles<R: Read + Seek> {
    pub(super) reader: IndexReader<R>,
    pub(super) count: usize,
    pub(super) visited: usize,
}

/// The expected signature of SqPack Files
const SQPACK_SIGNATURE: [u8; 6] = [0x53,0x71,0x50,0x61,0x63,0x6b];

/// The expected type ID of SqPack index files
const SQPACK_INDEX_TYPE: u8 = 2;

/// The offset after the sqpack header to find info about the files in the index file.
const FILE_INFO_OFFSET: u64 = 0x8;

/// The offset relative to `FILE_INFO_OFFSET` to find the length of the files section
const FILE_LENGTH_OFFSET: u64 = 0x4;

impl<R: Read + Seek> IndexReader<R> {

    /// Accepts a `Read + Seek` and wraps an `IndexReader` around it.
    ///
    /// # Returns
    /// `Ok(IndexReader)` if `inner` was a `Read` over a SqPack index file
    /// `Err(...)` if an I/O error occurred or if `inner` was not a `Read` over a SqPack index file.
    pub fn new(inner: R) -> SqResult<Self> {
        let mut inner = inner;
        inner.seek(SeekFrom::Start(0))?;
        let mut sq_sig_buffer = [0; 6];
        inner.read_exact(&mut sq_sig_buffer)?;
        if sq_sig_buffer.as_ref() == SQPACK_SIGNATURE.as_ref() {
            inner.seek(SeekFrom::Start(0x14))?;
            let sqtype = inner.read_u8()?;
            if sqtype == SQPACK_INDEX_TYPE {
                Ok(IndexReader{inner})
            } else {
                Err(SqpackError::ReaderIsNotIndex)
            }
        } else {
            Err(SqpackError::ReaderIsNotSqPack)
        }
    }

    /// Reads the header length from the internal reader.
    ///
    /// # Returns
    /// The header length wrapped in a result.
    fn header_length(&mut self) -> SqResult<u32> {
        self.inner.seek(SeekFrom::Start(0x0c))?;
        Ok(self.inner.read_u32::<LE>()?)
    }

    /// Reads the underlying reading for the offset in the .index file to the files section data.
    fn files_offset(&mut self) -> SqResult<u32> {

        let header_len = self.header_length()?;
        self.inner.seek(SeekFrom::Start(header_len as u64 + FILE_INFO_OFFSET))?;
        let val = self.inner.read_u32::<LE>()?;
        Ok(val)
    }

    /// Reads the underlying reader for the length in bytes of the files section
    fn files_length(&mut self) -> SqResult<u32> {
        let header_len = self.header_length()?;
        self.inner.seek(SeekFrom::Start(header_len as u64 + FILE_INFO_OFFSET + FILE_LENGTH_OFFSET))?;
        let length = self.inner.read_u32::<LE>()?;
        Ok(length)
    }

    /// Reads the number of files specified by this index file
    pub fn files_count(&mut self) -> SqResult<usize> {
        self.files_length().map(|len| (len >> 4) as usize)
    }

    /// Consumes the reader, yielding an iterator over the files present in the index.
    pub fn files(self) -> SqResult<IndexFiles<R>> {
        let mut slf = self;
        let count = slf.files_count()?;
        slf.seek_files()?;
        Ok(IndexFiles{reader: slf, count, visited: 0})
    }

    /// Seeks the reader to the files segment.
    pub fn seek_files(&mut self) -> SqResult<()> {
        let offset = self.files_offset()?;
        self.inner.seek(SeekFrom::Start(offset as u64))?;
        Ok(())
    }

    /// Reads a file entry from the index file. The underlying reader must be at a file,
    /// or you may get corrupted data. See [`seek_files`](method.seek_files.html). After execution,
    /// the underlying cursor is at the next file, if it exists.
    pub fn read_file(&mut self) -> SqResult<IndexFileEntry> {
        let file_hash = self.inner.read_u32::<LE>()?;
        let folder_hash = self.inner.read_u32::<LE>()?;
        let offset = self.inner.read_u32::<LE>()?;
        let dat_file = ((offset & 0x7) >> 1) as u8;
        let data_offset = ((offset & 0xfffffff8) << 3) as u32;
        self.inner.read_u32::<LE>()?;
        Ok(IndexFileEntry{file_hash, folder_hash, dat_file, data_offset})
    }

}

impl<R: Read + Seek> Iterator for IndexFiles<R> {
    type Item = SqResult<IndexFileEntry>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.visited < self.count {
            self.visited += 1;
            Some(self.reader.read_file())
        } else {
            None
        }
    }
}


