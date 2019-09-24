use std::io::{Read, Seek, SeekFrom, BufRead, BufReader};
use std::fs::File;
use SqResult;
use crate::byteorder::{ReadBytesExt, LE};
use io::index::IndexFile;

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

pub struct IndexReader<R>
    where R: Read + Seek + Sized
{
    inner: BufReader<R>,
}

pub struct IndexFiles<R: Read + Seek + Sized> {
    pub reader: IndexReader<R>,
}

impl<R: Read + Seek + Sized> IndexReader<R> {

    pub fn new(inner: R) -> Self {
        IndexReader{ inner: BufReader::new(inner) }
    }

    pub fn header_length(&mut self) -> SqResult<u32> {
        self.inner.seek(SeekFrom::Start(0x0c))?;
        Ok(self.inner.read_u32::<LE>()?)
    }

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
    type Item = IndexFile;
    fn next(&mut self) -> Option<Self::Item> {
//        self.reader.
        unimplemented!()
    }
}


