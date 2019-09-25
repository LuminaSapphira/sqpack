pub use crate::hash_consts::{FFXIV_CRC_TABLE, FFXIV_SEED};

/// Computes a hash from a seed, a CRC table, a buffer, the start index of the buffer,
/// and the size of the slice to hash. Typically you would not need to call this, for most uses
/// call [`SqPath::sq_index_hash`](../sqpath/struct.SqPath.html#method.sq_index_hash) instead. If
/// `lower` is true, the hash will be calculated as if all uppercase alphabet characters were lowercase.
///
/// # Examples
/// ```
/// use sqpack::hash::{compute_with_seed, FFXIV_SEED, FFXIV_CRC_TABLE};
/// let file_hash = compute_with_seed(FFXIV_SEED, &FFXIV_CRC_TABLE, b"BGM_System_Title.scd", 0, b"BGM_System_Title.scd".len() as u32, true);
/// assert_eq!(file_hash, 0xE3B71579);
/// ```
pub fn compute_with_seed(seed: u32, table: &[u32], buffer: &[u8], start: u32, size: u32, lower: bool) -> u32 {
    let mut crc = seed;
    for i in start..size+start {
        let mut b = buffer[i as usize];
        b ^= if lower && b >= 0x41 && b <= 0x5a { 0x20 } else { 0x00 };
        crc = (crc >> 8) ^ table[(b as u8 ^ crc as u8) as usize];
    }
    crc
}

/// Computes a string's hash with the default seed used for FFXIV. Unless you're re-implementing
/// certain parts of this library, you shouldn't need to use this. Use [`SqPath::sq_index_hash`](../sqpath/struct.SqPath.html#method.sq_index_hash)
/// instead. This function will simply convert the string to bytes and pass it to [`compute_with_seed`](fn.compute_with_seed.html).
///
/// # Examples
/// ```
/// use sqpack::hash::compute_str;
/// let file_hash = compute_str("bgm_system_title.scd");
/// assert_eq!(file_hash, 0xE3B71579);
/// ```
pub fn compute_str<S: AsRef<str> + ?Sized>(val: &S) -> u32 {
    compute_with_seed(FFXIV_SEED, &FFXIV_CRC_TABLE, val.as_ref().as_bytes(), 0, val.as_ref().as_bytes().len() as u32, false)
}

/// Computes a string's hash with the default seed used for FFXIV. Unless you're re-implementing
/// certain parts of this library, you shouldn't need to use this. Use [`SqPath::sq_index_hash`](../sqpath/struct.SqPath.html#method.sq_index_hash)
/// instead. This function will simply convert the string to bytes and pass it to [`compute_with_seed`](fn.compute_with_seed.html).
///
/// # Examples
/// ```
/// use sqpack::hash::compute_str_lower;
/// let file_hash = compute_str_lower("BGM_System_Title.scd");
/// assert_eq!(file_hash, 0xE3B71579);
/// ```
pub fn compute_str_lower<S: AsRef<str> + ?Sized>(val: &S) -> u32 {
    compute_with_seed(FFXIV_SEED, &FFXIV_CRC_TABLE, val.as_ref().as_bytes(), 0, val.as_ref().as_bytes().len() as u32, true)
}

#[cfg(test)]
mod hash_tests {
    use hash;

    #[test]
    fn case_eq() {
        assert_eq!(hash::compute_str_lower("music/ffxiv/BGM_System_Title.scd"), hash::compute_str_lower("music/ffxiv/bgm_system_title.scd"));
        assert_eq!(hash::compute_str_lower("music/ffxiv/BGM_System_Title.scd"), hash::compute_str("music/ffxiv/bgm_system_title.scd"));
        assert_ne!(hash::compute_str("music/ffxiv/BGM_System_Title.scd"), hash::compute_str("music/ffxiv/bgm_system_title.scd"));
        assert_ne!(hash::compute_str("music/ffxiv/BGM_System_Title.scd"), hash::compute_str_lower("music/ffxiv/bgm_system_title.scd"));
    }

    #[test]
    fn test_hash_file_name() {
        assert_eq!(hash::compute_str("bgm_system_title.scd"), 0xE3B71579);
        assert_eq!(hash::compute_str_lower("BGM_System_Title.scd"), 0xE3B71579);
    }

    #[test]
    fn test_hash_folder_name() {
        assert_eq!(hash::compute_str("music/ffxiv"), 0x0AF269D6);
        assert_eq!(hash::compute_str_lower("music/ffxiv"), 0x0AF269D6);
    }

}