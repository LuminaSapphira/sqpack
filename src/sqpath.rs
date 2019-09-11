
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

#[cfg(test)]
mod sqpath_tests {
    use SqPath;
    use sqpath::SqPathBuf;

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
}