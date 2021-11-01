extern crate md5;
extern crate sqpack;
extern crate walkdir;

const FFXIV_SQPACK_PATH: &'static str = "FFXIV_SQPACK_PATH";

use sqpack::{io::index::IndexReader, SqPath};
use std::{
    collections::HashMap,
    env,
    fs::{File, FileType},
    io::{BufRead, BufReader, Read},
};

fn get_env_vars() -> HashMap<String, String> { env::vars().collect() }

#[test]
fn environment_vars_correct() {
    let var_map = get_env_vars();
    assert!(var_map.contains_key(FFXIV_SQPACK_PATH));
}

#[test]
fn index_reader_iterators() {
    let mut indices = 0;
    let path = &get_env_vars()[FFXIV_SQPACK_PATH];
    walkdir::WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter(|entry_res| entry_res.is_ok())
        .map(|entry_res| entry_res.unwrap())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "index")
                .unwrap_or(false)
        })
        .map(|a| a.into_path())
        .for_each(|index_path| {
            indices += 1;
            let index_file = File::open(index_path).expect("Failed to open index file for reading");
            let mut reader = IndexReader::new(index_file).expect("Failed to create index reader");
            for res in reader.files().expect("Failed to create files iterator") {
                res.expect("Failed to read file");
            }
            for fol in reader.folders().expect("Failed to create folders iterator") {
                fol.expect("Failed to read folder");
            }
            for res in reader
                .files()
                .expect("Failed to create a files iterator for the second time")
            {
                res.expect("Failed to read file in 2nd iterator");
            }
            let mut folders_info = vec![];
            for fol in reader
                .folders()
                .expect("Failed to create a folders iterator for the second time")
            {
                let fol = fol.expect("Failed to read folder in 2nd iter");
                folders_info.push(fol);
            }
            for fol_info in folders_info {
                let mut count: u32 = 0;
                for contents in reader
                    .folder_contents(&fol_info)
                    .expect("Failed to get folder contents")
                {
                    contents.expect("Failed to read file in folder contents");
                    count += 1;
                }
                assert_eq!(fol_info.files_count, count);
            }
        });
    assert_ne!(indices, 0, "Didn't read any indices");
}

#[test]
fn open_file() {
    use sqpack::io::dat::SqFile;

    let sqpack = &get_env_vars()[FFXIV_SQPACK_PATH];
    let sqpath = "music/ffxiv/BGM_System_Title.scd";
    SqFile::open_sqpath(sqpath, sqpack).expect("Opening file");
}

#[test]
fn read_file() {
    use sqpack::io::dat::SqFile;

    let sqpack = &get_env_vars()[FFXIV_SQPACK_PATH];
    let sqpath = "music/ffxiv/BGM_System_Title.scd";
    let mut sqfile = SqFile::open_sqpath(sqpath, sqpack).expect("Opening file");
    let mut data = Vec::with_capacity(sqfile.total_size());
    sqfile.read_to_end(&mut data).expect("Reading");

    let expected: [u8; 16] = [
        0x43, 0x51, 0x52, 0x41, 0xA8, 0xE7, 0x8E, 0xCC, 0xD5, 0xE1, 0xB3, 0x3A, 0xBE, 0x89, 0xDB,
        0xCC,
    ];
    let digest = md5::compute(data).0;
    assert_eq!(expected, digest, "File not equal to expected file!")
}
