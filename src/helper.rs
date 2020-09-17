
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub fn indode_of_path(p: &OsStr) -> u64 {
    let mut hasher = DefaultHasher::new();
    p.hash(&mut hasher);
    hasher.finish()
}