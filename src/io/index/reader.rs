use std::io::{Read, Seek, SeekFrom, BufReader};
use ::{SqResult, SqpackError};
use crate::byteorder::{ReadBytesExt, LE};
use io::index::IndexFileEntry;

/// A buffered reader that reads index files from a wrapped `Read` instance
pub struct IndexReader<R>
    where R: Read + Seek + Sized
{
    inner: BufReader<R>,
}

/// An iterator struct over the files present in the passed IndexReader
pub struct IndexFiles<R: Read + Seek + Sized> {
    pub reader: IndexReader<R>,
}

/// The expected signature of SqPack Files
const SQPACK_SIGNATURE: [u8; 6] = [0x53,0x71,0x50,0x61,0x63,0x6b];

/// The expected type ID of SqPack index files
const SQPACK_INDEX_TYPE: u8 = 2;

impl<R: Read + Seek + Sized> IndexReader<R> {

    /// Accepts a `Read + Seek` and wraps an `IndexReader` around it.
    ///
    /// # Returns
    /// `Ok(IndexReader)` if `inner` was a `Read` over a SqPack index file
    /// `Err(...)` if an I/O error occurred or if `inner` was not a `Read` over a SqPack index file.
    pub fn new(inner: R) -> SqResult<Self> {
        let mut buf = BufReader::new(inner);
        let mut sq_sig_buffer = [0; 6];
        buf.read_exact(&mut sq_sig_buffer)?;
        if sq_sig_buffer.as_ref() == SQPACK_SIGNATURE.as_ref() {
            buf.seek(SeekFrom::Start(0x14))?;
            let sqtype = buf.read_u8()?;
            if sqtype == SQPACK_INDEX_TYPE {
                Ok(IndexReader{inner: buf})
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

    ///
    pub fn files_len(&mut self) -> SqResult<usize> {

    }

    /// Consumes the reader, yielding an iterator over the files present in the index.
    pub fn files(self) -> SqResult<IndexFiles<R>> {
        let mut s = self;
        let len = s.header_length()?;


        let mut inner = s.inner;
//        unimplemented!();
//        IndexFiles { reader: inner };
        unimplemented!()
    }

}

impl<R: Read + Seek + Sized> Iterator for IndexFiles<R> {
    type Item = IndexFileEntry;
    fn next(&mut self) -> Option<Self::Item> {
//        self.reader.
        unimplemented!()
    }
}


