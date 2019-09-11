//use std::path::Path;

//use crate::{SqpackError, SqPath};

pub struct SqFile;

//
//impl SqFile {
//
//    /// Opens an SqFile for reading, where `sq_path` is the "in-data-files" path,
//    /// and `sqpack_path` is the location of the data files.
//    ///
//    /// # Examples
//    /// ```
//    /// use sqpack::SqFile;
//    /// SqFile::open("music/ffxiv/BGM_System_Title.scd", "C:\\Program Files (x86)\\SquareEnix\\FINAL FANTASY XIV - A Realm Reborn\\game\\sqpack").expect("Unable to open or find file");
//    /// ```
//    pub fn open<'a, P, O>(sq_path: &'a P, sqpack_path: &O) -> Result<&'a SqFile, SqpackError>
//        where P: AsRef<SqPath> + ?Sized,
//              O: AsRef<Path> + ?Sized
//    {
//        Err(SqpackError::SqFileNotFound)
//    }
//}