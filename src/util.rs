extern crate "rust-crypto" as rust_crypto;

use std::io::{fs, File};
use std::io::fs::PathExtensions;
use std::io::fs::Directories;
use std::collections::HashMap;
use std::vec::Vec;
use self::rust_crypto::digest::Digest;
use self::rust_crypto::sha1::Sha1;

pub struct Metadata {
    pub path: String,
    pub size: u64,
    pub modified: u64
}

pub struct MetadataReader {
    directory_iterator: Directories
}

impl MetadataReader {
    pub fn new(path: &Path) -> MetadataReader {
        let iterator = fs::walk_dir(path).unwrap();
        MetadataReader { directory_iterator: iterator }
    }
}

impl Iterator<Metadata> for MetadataReader {

    fn next(&mut self) -> Option<Metadata> {
        let path = match self.directory_iterator.next() {
            Some(p) => p,
            None => return None
        };
        let stat = path.stat().unwrap();
        Some(Metadata { path: from_str(path.as_str().unwrap()).unwrap(), size: stat.size, modified: stat.modified })
    }
}

pub fn examine_files(directory_path: &Path, map: &mut HashMap<(u64,u64), Vec<String>>) {

    let entries = fs::readdir(directory_path);

    if !entries.is_ok() {
        return;
    }

    for entry in entries.unwrap().iter() {
        if entry.is_dir() {
            examine_files(entry, map);
        } else {
            let path = entry.as_str().unwrap();
            let stat = entry.stat();
            if stat.is_ok() {
                let unwrapped_stat = stat.unwrap();
                let key = (unwrapped_stat.size, unwrapped_stat.modified);
                if !map.contains_key(&key) {
                    map.insert(key, Vec::new());
                }
                map.get_mut(&key).push(path.to_string());
            }
        }
    }
}

pub fn get_file_checksum(file_path: &Path) -> String {

    let mut buffer = [0u8, ..4096];
    let mut file = File::open(file_path);
    let mut sha1 = Sha1::new();

    loop {
        match file.read(buffer) {
            Ok(bytes) => { sha1.input(buffer.slice(0, bytes)) }
            Err(_) => { break; }
        }
    }

    return sha1.result_str();

}
