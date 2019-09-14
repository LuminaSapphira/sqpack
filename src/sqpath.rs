use std::borrow::Borrow;
use crate::{
    hash,
};

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
/// A representation of a location within the FFXIV data files. This is an
/// **unsized** type, so it must always be behind a reference such as & or Box.
/// Use SqPathBuf for the Owned/Sized/Allocated variant.
pub struct SqPath {
    inner: str
}

impl SqPath {
    /// Creates a new borrowed SqPath from a str-like input reference
    ///
    /// # Examples
    /// ```
    /// use sqpack::sqpath::SqPathBuf;
    /// use sqpack::SqPath;
    ///
    /// // from an &str
    /// let a = SqPath::new("testing");
    ///
    /// // from a String
    /// let s = String::from("testing");
    /// let b = SqPath::new(&s);
    ///
    /// assert_eq!(a, b)
    /// ```
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &SqPath {
        // Use of unsafe follows same format as std::path::Path for unsized type
        unsafe { &*(s.as_ref() as *const str as *const SqPath) }
    }

    /// Gets the SqIndexPath of the file. This struct allows you to locate
    /// a specific file within the index, as the index files are all encoded
    /// based on a specific hash of the file and folder name.
    ///
    /// # Returns
    /// `Some(...)` if the path was a valid SqIndex path, `None` otherwise. Note:
    /// this does not verify if the file is in the Sqpack, just if the path was well-formed.
    pub fn get_sq_index_path(&self) -> Option<SqIndexPath> {
        let path = &self.inner;
        let last_part = path.rfind("/");
        last_part.map(|index| SqIndexPath {
            folder_hash: hash::compute_str_lower(&path[0..index]),
            file_hash: hash::compute_str_lower(&path[index + 1..])
        })
    }
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Debug, Hash, Clone)]
/// An owned, sized representation of a location within the FFXIV data files.
pub struct SqPathBuf {
    inner: String
}

impl SqPathBuf {
    /// Creates a new owned & allocated SqPathBuf from a str-like input reference
    ///
    /// # Examples
    /// ```
    /// use sqpack::sqpath::SqPathBuf;
    /// // from an &str
    /// let a = SqPathBuf::new("testing");
    ///
    /// // from a String ref
    /// let s = String::from("testing");
    /// let b = SqPathBuf::new(&s);
    ///
    /// assert_eq!(a, b)
    /// ```
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> SqPathBuf {
        SqPathBuf { inner: String::from(s.as_ref()) }
    }
}

/// A simple struct that names the parts of an Sqpack Index file path
pub struct SqIndexPath {
    pub folder_hash: u32,
    pub file_hash: u32,
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum FileType {
    Common,
    BGCommon,
    BG,
    Cut,
    Chara,
    Shader,
    UI,
    Sound,
    VFX,
    UIScript,
    EXD,
    GameScript,
    Music,
    SqpackTest,
    Debug,
}



impl AsRef<SqPath> for str {
    fn as_ref(&self) -> &SqPath {
        SqPath::new(self)
    }
}

impl AsRef<SqPath> for SqPath {
    fn as_ref(&self) -> &SqPath { self }
}

impl AsRef<SqPath> for String {
    fn as_ref(&self) -> &SqPath { SqPath::new(self.as_str()) }
}

impl AsRef<SqPath> for SqPathBuf {
    fn as_ref(&self) -> &SqPath { self.inner.as_ref() }
}

impl ToOwned for SqPath {
    type Owned = SqPathBuf;
    fn to_owned(&self) -> Self::Owned {
        SqPathBuf::new(&self.inner)
    }
}

impl Borrow<SqPath> for SqPathBuf {
    fn borrow(&self) -> &SqPath {
        SqPath::new(&self.inner)
    }
}

#[cfg(test)]
mod sqpath_tests {
    use SqPath;
    use sqpath::SqPathBuf;
    use std::borrow::Borrow;

    #[test]
    fn basic_sqpath() {
        let iref = &SqPath::new("asdasd").inner;
        assert_eq!(iref, "asdasd");
    }

    #[test]
    fn sqpath_as_refs() {
        let a: &SqPath = "uwu".as_ref();
        let b: &SqPath = a.as_ref();
        let s = String::from("uwu");
        let c: &SqPath = s.as_ref();
        let sqpb = SqPathBuf::new("uwu");
        let d: &SqPath = sqpb.as_ref();
        assert_eq!(&a.inner, "uwu");
        assert_eq!(&b.inner, "uwu");
        assert_eq!(&c.inner, "uwu");
        assert_eq!(&d.inner, "uwu");
    }

    #[test]
    fn basic_sqpathbuf() {
        let sqpb = SqPathBuf::new("uwu");
        assert_eq!(sqpb.inner, "uwu");
    }

    #[test]
    fn new_params_any() {
        SqPathBuf::new("uwu");
        let s = String::from("uwu");
        SqPathBuf::new(&s);
    }

    #[test]
    fn sq_index_path() {
        let sq_path = SqPath::new("music/ffxiv/BGM_System_Title.scd");
        let sq_index_path = sq_path.get_sq_index_path().expect("Path was not well formed");
        assert_eq!(sq_index_path.folder_hash, 0x0AF269D6);
        assert_eq!(sq_index_path.file_hash, 0xE3B71579)
    }

    #[test]
    fn to_owned_and_borrow() {
        let sqpath = SqPath::new("uwu");
        let a: SqPathBuf = sqpath.to_owned();
        assert_eq!(a.inner, sqpath.inner);

        let x: &SqPath = a.borrow();
        assert_eq!(x.inner, a.inner);
    }
}