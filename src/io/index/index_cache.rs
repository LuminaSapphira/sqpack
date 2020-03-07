use io::index::IndexReader;
use std::collections::HashMap;
use std::io::{Read, Seek};
use crate::error::SqResult;
use sqpath::SqIndexHash;

/// An in-memory cache of a single .index file. Recommended for reading many files all from the
/// same index.
#[derive(Clone, PartialEq, Debug)]
pub struct IndexCache {
    folders: HashMap<u32, IndexFolderEntry>,
}

/// A folder entry within the index cache. Files can be found within.
#[derive(Clone, PartialEq, Debug)]
pub struct IndexFolderEntry {
    pub folder_hash: u32,
    pub(self) files: HashMap<u32, IndexFileEntry>,
}

/// A file entry within the index cache. Can be used to locate the file data within the .dat files.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct IndexFileEntry {

    /// The hash of this entry
    pub path_hash: SqIndexHash,

    /// Where in the .dat file the data may be found
    pub data_offset: u32,

    /// Which .dat file the file may be found in. I.
    pub dat_file: u8,
}

impl IndexCache {
    /// Creates a new cache for an index file from a mutable reference to an `IndexReader`.
    pub fn from_reader<R: Read + Seek>(reader: &mut IndexReader<R>) -> SqResult<IndexCache> {
        let folders_res = reader.folders()?.collect::<Vec<_>>();
        let mut folders = Vec::with_capacity(folders_res.len());
        for folder_res in folders_res {
            folders.push(folder_res?);
        }
        let folders = folders;

        let mut folder_map = HashMap::new();

        for folder in folders {
            let mut files = HashMap::with_capacity(folder.files_count as usize);
            for file in reader.folder_contents(&folder)? {
                let file = file?;
                files.insert(file.path_hash.file_hash, file);
            }
            folder_map.insert(
                folder.folder_hash,
                IndexFolderEntry {
                    folder_hash: folder.folder_hash,
                    files,
                },
            );
        }
        Ok(IndexCache {
            folders: folder_map,
        })
    }
}
