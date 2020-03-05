use crate::byteorder::{ReadBytesExt, LE};
use crate::seek_bufread::BufReader;
use io::index::IndexFileEntry;
use std::io::{Read, Seek, SeekFrom};
use crate::error::{SqResult, SqpackError};

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Hash, Default)]
struct CachedInfo {
    header_length: Option<u32>,
    files_offset: Option<u32>,
    folders_offset: Option<u32>,
    files_length: Option<u32>,
    folders_length: Option<u32>,
}

/// A buffered reader that reads index files from a wrapped `Read` instance
pub struct IndexReader<R>
where
    R: Read + Seek,
{
    pub(self) inner: BufReader<R>,
    cache: CachedInfo,
}

/// An iterator struct over the files present in the passed IndexReader
pub struct IndexFiles<'a, R: Read + Seek> {
    pub(self) reader: &'a mut IndexReader<R>,
    pub(self) count: usize,
    pub(self) visited: usize,
}

/// An iterator over the folders present in the passed IndexReader
pub struct IndexFolders<'a, R: Read + Seek> {
    pub(self) reader: &'a mut IndexReader<R>,
    pub(self) count: usize,
    pub(self) visited: usize,
}

/// Contains info about a folder in an index file, used for reading lists of files
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct IndexFolderInfo {
    /// The hash of the folder name, can be used to find files in an `IndexCache`
    pub folder_hash: u32,
    files_offset: u32,
    pub files_count: u32,
}

/// An iterator over the contents of a specific folder in the index
pub struct IndexFolderContents<'a, R: Read + Seek> {
    pub(self) reader: &'a mut IndexReader<R>,
    pub(self) files_count: u32,
    pub(self) files_visited: u32,
}

/// The expected signature of SqPack Files
const SQPACK_SIGNATURE: [u8; 6] = [0x53, 0x71, 0x50, 0x61, 0x63, 0x6b];

/// The expected type ID of SqPack index files
const SQPACK_INDEX_TYPE: u8 = 2;

/// The offset after the sqpack header to find info about the files in the index file.
const FILE_INFO_OFFSET: u64 = 0x8;

/// The offset relative to `FILE_INFO_OFFSET` to find the length of the files section
const FILE_LENGTH_OFFSET: u64 = 0x4;

/// The offset relative to the sqpack header end to find info about the folders in the index file
const FOLDER_INFO_OFFSET: u64 = 0xE4;

/// The offset relative to `FOLDER_INFO_OFFSET` to find the length of the folders section
const FOLDER_LENGTH_OFFSET: u64 = 0x4;

impl<R: Read + Seek> IndexReader<R> {
    /// Accepts a `Read + Seek` and wraps an `IndexReader` around it.
    ///
    /// # Returns
    /// `Ok(IndexReader)` if `inner` was a `Read` over a SqPack index file
    /// `Err(...)` if an I/O error occurred or if `inner` was not a `Read` over a SqPack index file.
    pub fn new(inner: R) -> SqResult<Self> {
        Self::with_capacity(16384, inner)
    }

    /// Creates and `IndexReader` with the specified capacity. See `IndexReader::new`.
    pub fn with_capacity(cap: usize, inner: R) -> SqResult<Self> {
        let mut inner = BufReader::with_capacity(cap, inner);
        inner.seek(SeekFrom::Start(0))?;
        let mut sq_sig_buffer = [0; 6];
        inner.read_exact(&mut sq_sig_buffer)?;
        if sq_sig_buffer.as_ref() == SQPACK_SIGNATURE.as_ref() {
            inner.seek(SeekFrom::Start(0x14))?;
            let sqtype = inner.read_u8()?;
            if sqtype == SQPACK_INDEX_TYPE {
                Ok(IndexReader {
                    inner,
                    cache: Default::default(),
                })
            } else {
                Err(SqpackError::ReaderIsNotIndex)
            }
        } else {
            Err(SqpackError::ReaderIsNotSqPack)
        }
    }

    /// Reads the header length from the internal reader. The reader position is not guaranteed to be
    /// the same after calling.
    ///
    /// # Returns
    /// The header length wrapped in a result.
    fn header_length(&mut self) -> SqResult<u32> {
        if let Some(len) = self.cache.header_length {
            Ok(len)
        } else {
            self.inner.seek(SeekFrom::Start(0x0c))?;
            let len = self.inner.read_u32::<LE>()?;
            self.cache.header_length = Some(len);
            Ok(len)
        }
    }

    /// Reads the underlying reading for the offset in the .index file to the files section data.
    /// The reader position is not guaranteed to be the same after calling.
    fn files_offset(&mut self) -> SqResult<u32> {
        if let Some(offset) = self.cache.files_offset {
            Ok(offset)
        } else {
            let header_len = self.header_length()?;
            self.inner
                .seek(SeekFrom::Start(header_len as u64 + FILE_INFO_OFFSET))?;
            let val = self.inner.read_u32::<LE>()?;
            self.cache.files_offset = Some(val);
            Ok(val)
        }
    }

    /// Reads the underlying reader for the length in bytes of the files section.
    /// The reader position is not guaranteed to be the same after calling.
    fn files_length(&mut self) -> SqResult<u32> {
        if let Some(len) = self.cache.files_length {
            Ok(len)
        } else {
            let header_len = self.header_length()?;
            self.inner.seek(SeekFrom::Start(
                header_len as u64 + FILE_INFO_OFFSET + FILE_LENGTH_OFFSET,
            ))?;
            let length = self.inner.read_u32::<LE>()?;
            self.cache.files_length = Some(length);
            Ok(length)
        }
    }

    /// Reads the underlying reader for the offset to the folders section.
    /// The reader position is not guaranteed to be the same after calling.
    fn folders_offset(&mut self) -> SqResult<u32> {
        if let Some(offset) = self.cache.folders_offset {
            Ok(offset)
        } else {
            let header_len = self.header_length()?;
            self.inner
                .seek(SeekFrom::Start(header_len as u64 + FOLDER_INFO_OFFSET))?;
            let val = self.inner.read_u32::<LE>()?;
            self.cache.folders_offset = Some(val);
            Ok(val)
        }
    }

    /// Reads the underlying reader for the length in bytes of the folders section.
    /// The reader position is not guaranteed to be the same after calling.
    fn folders_length(&mut self) -> SqResult<u32> {
        if let Some(len) = self.cache.folders_length {
            Ok(len)
        } else {
            let header_len = self.header_length()?;
            self.inner.seek(SeekFrom::Start(
                header_len as u64 + FOLDER_INFO_OFFSET + FOLDER_LENGTH_OFFSET,
            ))?;
            let length = self.inner.read_u32::<LE>()?;
            self.cache.folders_length = Some(length);
            Ok(length)
        }
    }

    /// Reads the number of files specified by this index file
    pub fn files_count(&mut self) -> SqResult<usize> {
        self.files_length().map(|len| (len >> 4) as usize)
    }

    /// Reads the number of folders specified by this index file
    pub fn folders_count(&mut self) -> SqResult<usize> {
        self.folders_length().map(|len| (len >> 4) as usize)
    }

    /// Creates an iterator over the files present in the index.
    pub fn files(&mut self) -> SqResult<IndexFiles<R>> {
        let count = self.files_count()?;
        self.seek_files()?;
        Ok(IndexFiles {
            reader: self,
            count,
            visited: 0,
        })
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
    pub fn read_file_entry(&mut self) -> SqResult<IndexFileEntry> {
        let file_hash = self.inner.read_u32::<LE>()?;
        let folder_hash = self.inner.read_u32::<LE>()?;
        let offset = self.inner.read_u32::<LE>()?;
        let dat_file = ((offset & 0x7) >> 1) as u8;
        let data_offset = ((offset & 0xfffffff8) << 3) as u32;
        self.inner.read_u32::<LE>()?;
        Ok(IndexFileEntry {
            file_hash,
            folder_hash,
            dat_file,
            data_offset,
        })
    }

    /// Seeks the reader to the folders segment
    pub fn seek_folders(&mut self) -> SqResult<()> {
        let offset = self.folders_offset()?;
        self.inner.seek(SeekFrom::Start(offset as u64))?;
        Ok(())
    }

    /// Reads a folder entry from the index file. The underlying reader must be at a folder,
    /// or you may get corrupted data. See [`seek_folders`](method.seek_folders.html). After
    /// execution, the underlying cursor is at the next file, if it exists.
    pub fn read_folder_entry(&mut self) -> SqResult<IndexFolderInfo> {
        let folder_hash = self.inner.read_u32::<LE>()?;
        let files_offset = self.inner.read_u32::<LE>()?;
        let files_size = self.inner.read_u32::<LE>()?;
        self.inner.seek(SeekFrom::Current(4))?;
        let files_count = files_size >> 4;
        Ok(IndexFolderInfo {
            folder_hash,
            files_offset,
            files_count,
        })
    }

    /// Creates an iterator over the folder entries of the reader.
    /// *Note*: you cannot use this method to obtain file info. See [`folder_contents`](method.folder_contents.html).
    pub fn folders(&mut self) -> SqResult<IndexFolders<R>> {
        let count = self.folders_count()?;
        self.seek_folders()?;
        Ok(IndexFolders {
            reader: self,
            count,
            visited: 0,
        })
    }

    /// Seeks the underlying reader to the location of the specified folder's contents
    pub fn seek_folder_contents(&mut self, info: &IndexFolderInfo) -> SqResult<()> {
        self.inner.seek(SeekFrom::Start(info.files_offset as u64))?;
        Ok(())
    }

    /// Creates an iterator over the contents of the folder identified by `folder_info`
    pub fn folder_contents(
        &mut self,
        folder_info: &IndexFolderInfo,
    ) -> SqResult<IndexFolderContents<R>> {
        self.seek_folder_contents(folder_info)?;
        Ok(IndexFolderContents {
            reader: self,
            files_count: folder_info.files_count,
            files_visited: 0,
        })
    }
}

impl<'a, R: Read + Seek> Iterator for IndexFolderContents<'a, R> {
    type Item = SqResult<IndexFileEntry>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.files_visited < self.files_count {
            self.files_visited += 1;
            Some(self.reader.read_file_entry())
        } else {
            None
        }
    }
}

impl<'a, R: Read + Seek> Iterator for IndexFiles<'a, R> {
    type Item = SqResult<IndexFileEntry>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.visited < self.count {
            self.visited += 1;
            Some(self.reader.read_file_entry())
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let c = self.count - self.visited;
        (c, Some(c))
    }
}

impl<'a, R: Read + Seek> Iterator for IndexFolders<'a, R> {
    type Item = SqResult<IndexFolderInfo>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.visited < self.count {
            self.visited += 1;
            Some(self.reader.read_folder_entry())
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let c = self.count - self.visited;
        (c, Some(c))
    }
}
