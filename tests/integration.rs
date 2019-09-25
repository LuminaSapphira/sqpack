extern crate sqpack;

const FFXIV_SQPACK_PATH: &'static str = "FFXIV_SQPACK_PATH";

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead};
use sqpack::io::index::IndexReader;
use sqpack::SqPath;

fn get_env_vars() -> HashMap<String, String> {
    env::vars().collect()
}

#[test]
fn environment_vars_correct() {
    let var_map = get_env_vars();
    assert!(var_map.contains_key(FFXIV_SQPACK_PATH));
}

#[test]
fn index_iterator() {
    let path = &get_env_vars()[FFXIV_SQPACK_PATH];
    let sqpath = SqPath::new("music/ffxiv/BGM_System_Title.scd");
    let hash = sqpath.sq_index_hash().unwrap();

    let index_path = sqpath.sqpack_index_path(path).unwrap();
    let index_file = File::open(index_path).unwrap();
    let reader = IndexReader::new(index_file).unwrap();
    for res in reader.files().unwrap() {
        let entry = res.unwrap();
        println!("{}/{}", entry.folder_hash, entry.file_hash);
    }
}

//#[test]
//fn open_file() {
//    let path = &get_env_vars()[FFXIV_SQPACK_PATH];
//    sqpack::SqFile::open("music/ffxiv/BGM_System_Title.scd", path).unwrap();
//}
