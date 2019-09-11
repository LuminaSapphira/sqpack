extern crate sqpack;

const FFXIV_SQPACK_PATH: &'static str = "FFXIV_SQPACK_PATH";

use std::collections::HashMap;
use std::env;
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