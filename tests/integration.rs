extern crate sqpack;

const FFXIV_SQPACK_PATH: &'static str = "FFXIV_SQPACK_PATH";

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead};

fn get_env_vars() -> HashMap<String, String> {
    env::vars().collect()
}

#[test]
fn environment_vars_correct() {
    let var_map = get_env_vars();
    assert!(var_map.contains_key(FFXIV_SQPACK_PATH));
}

//#[test]
//fn open_file() {
//    let path = &get_env_vars()[FFXIV_SQPACK_PATH];
//    sqpack::SqFile::open("music/ffxiv/BGM_System_Title.scd", path).unwrap();
//}

#[test]
fn header_length() {
    let path = &get_env_vars()[FFXIV_SQPACK_PATH];
    let sq = sqpack::SqPath::new("music/ffxiv/BGM_System_Title.scd");
    let pb = sq.sqpack_index_path(path).unwrap();
    let file = File::open(pb).unwrap();
    let br = BufReader::new(file);
    let mut iread = sqpack::io::index::IndexReader::new(br);
    assert_eq!(iread.header_length().unwrap(), 0x400);
}