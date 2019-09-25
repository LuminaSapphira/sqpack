
use std::collections::HashMap;
use sqpath::{FileType, Expansion};

#[derive(Clone, PartialEq, Debug)]
pub struct IndexCache {
    folders: HashMap<u32, IndexFolderEntry>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct IndexFolderEntry {
    pub folder_hash: u32,
    files: HashMap<u32, IndexFileEntry>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct IndexFileEntry {
    pub folder_hash: u32,
    pub file_hash: u32,
    pub data_offset: u32,
    pub dat_file: u8,
}

impl IndexCache {

    pub fn new(file_type: FileType, game_expansion: Expansion) -> IndexCache {

        unimplemented!()

    }

}
