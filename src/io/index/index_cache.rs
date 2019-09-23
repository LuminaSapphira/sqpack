
use std::collections::HashMap;
use sqpath::{FileType, Expansion};

#[derive(Clone, PartialEq, Debug)]
pub struct IndexCache {
    folders: HashMap<u32, IndexFolder>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct IndexFolder {
    pub folder_hash: u32,
    files: HashMap<u32, IndexFile>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct IndexFile {
    pub folder_hash: u32,
    pub file_hash: u32,
    pub data_offset: u32,
    pub dat_file: u32,
}

impl IndexCache {

    pub fn new(file_type: FileType, game_expansion: Expansion) -> IndexCache {

        unimplemented!()

    }

}
