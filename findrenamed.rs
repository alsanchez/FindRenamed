use std::io::fs;
use std::io::File;
use std::collections::HashMap;
use std::os;

fn examine_files(directory_path: &Path, map: &mut HashMap<(u64,u64), String>) {

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
                map.insert((unwrapped_stat.size, unwrapped_stat.modified), path.to_string());
            }
        }
    }
}

fn compare_files(src_path: &Path, new_path: &Path) -> bool {
    let mut src_file = File::open(src_path);
    let mut new_file = File::open(new_path);
    let src_data = src_file.read_to_end();
    let new_data = new_file.read_to_end();

    src_data == new_data
}

fn main() {

    let args = os::args();

    if args.len() != 3 {
        println!("Usage: {} <original-directory> <new-directory>", args[0]);
        return;
    }

    let src_directory = Path::new(args[1].clone());
    let dst_directory = Path::new(args[2].clone());
        
    let mut src_map = HashMap::<(u64, u64), String>::new();
    examine_files(&src_directory, &mut src_map);
    let mut new_map = HashMap::<(u64, u64), String>::new();
    examine_files(&dst_directory, &mut new_map);
    for (key, value) in new_map.iter() {
        if src_map.contains_key(key) {
            let src_path = Path::new(src_map.get(key).clone());
            let new_path = Path::new(value.clone());
            if compare_files(&src_path, &new_path) {
                println!("{} was renamed to {}", 
                         src_path.path_relative_from(&src_directory).unwrap().as_str().unwrap(), 
                         new_path.path_relative_from(&dst_directory).unwrap().as_str().unwrap());
            }
        }
    }
}
