
#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
/// A representation of a location within the FFXIV data files. This is an
/// **unsized** type, so it must always be behind a reference such as & or Box.
/// Use SqPathBuf for the Owned/Sized/Allocated variant.
pub struct SqPath {
    inner: str
}

impl SqPath {
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &SqPath {
        // Use of unsafe follows same format as std::path::Path for unsized type
        unsafe { &*(s.as_ref() as *const str as *const SqPath) }
    }
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Debug, Hash, Clone)]
/// An owned, sized representation of a location within the FFXIV data files.
pub struct SqPathBuf {
    inner: String
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