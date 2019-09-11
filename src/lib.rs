
//! A simple crate for reading the data files of FFXIV.
//!
//! The main entry point for most use cases is the `SqPack` struct,
//! which accepts a path to the data files on the user's machine.
//! From an instance of this struct, you can open `Read` implementors
//! which return the decoded data.
//!
//! For more advanced usage, the internal mechanisms are exposed. Typically
//! one would start with a `SqPath` / `SqPathBuf`, pass that to a
//! function in the `io` module, along with a path to the data files, to
//! open `Read` streams.

pub mod sqpath;
pub mod io;

pub use io::{
    SqpackError,
    sqfile::SqFile
};
pub use sqpath::{
    SqPath
};