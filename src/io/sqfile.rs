use std::{
    fs::{
        File
    }
};
use SqPath;

pub struct SqFile {
    index_file: File,
    dat_file: Option<File>
}

impl SqFile {
    pub fn open<P: AsRef<SqPath>>(path: P) -> SqFile {
        let path = path.as_ref();
        unimplemented!()
    }
}