use std::{
    fs::File,
    path::Path,
};
use ::{SqPath, SqpackError};
use io::SqResult;
use io::index::IndexCache;

pub struct SqFile {
    dat_file: File,
}

impl SqFile {


    pub fn open<P, O>(sqpath: P, sqpack_path: O) -> SqResult<SqpackError>
        where P: AsRef<SqPath>, O: AsRef<Path>
    {

        


        unimplemented!()
    }

    pub fn open_with_index<P, O>(sqpath: P, index: &IndexCache, sqpack_path: O) -> SqResult<SqpackError>
        where P: AsRef<SqPath>, O: AsRef<Path>
    {
        let sqpath = sqpath.as_ref();
        let sqpack_path = sqpack_path.as_ref();



        unimplemented!()

    }
}
