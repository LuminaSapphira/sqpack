extern crate sqpack;
extern crate walkdir;

const FFXIV_SQPACK_PATH: &'static str = "FFXIV_SQPACK_PATH";

use sqpack::io::index::IndexReader;
use sqpack::SqPath;
use std::collections::HashMap;
use std::env;
use std::fs::{File, FileType};
use std::io::{BufRead, BufReader};

fn get_env_vars() -> HashMap<String, String> {
    env::vars().collect()
}

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

//#[test]
//fn open_file() {
//    let path = &get_env_vars()[FFXIV_SQPACK_PATH];
//    sqpack::SqFile::open("music/ffxiv/BGM_System_Title.scd", path).unwrap();
//}
