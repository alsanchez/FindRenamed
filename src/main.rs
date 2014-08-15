use std::collections::HashMap;
use std::os;
use std::vec::Vec;

mod util;
mod server;

fn compare_files(src_path: &Path, new_path: &Path) -> bool {

    return util::get_file_checksum(src_path) == util::get_file_checksum(new_path);
}

fn main() {

    let args = os::args();

    if args.len() == 2 && args[1] == "--start-server".to_string() {
        server::start_server();
        return;
    }

    if args.len() != 3 {
        println!("Usage: {} <original-directory> <new-directory>", args[0]);
        return;
    }

    let src_directory = Path::new(args[1].clone());
    let dst_directory = Path::new(args[2].clone());
        
    let mut src_map = HashMap::<(u64, u64), Vec<String>>::new();
    util::examine_files(&src_directory, &mut src_map);
    let mut new_map = HashMap::<(u64, u64), Vec<String>>::new();
    util::examine_files(&dst_directory, &mut new_map);
    for (key, paths) in new_map.iter() {
        if src_map.contains_key(key) {
            for (i, value) in paths.iter().enumerate() {
                let src_path = Path::new(src_map.get(key)[0].clone());
                let new_path = Path::new(value.clone());
                if compare_files(&src_path, &new_path) {

                    let mut file_path = dst_directory.clone();
                    file_path.push(src_path.path_relative_from(&src_directory).unwrap());

                    if src_path.path_relative_from(&src_directory).unwrap()
                        != new_path.path_relative_from(&dst_directory).unwrap() {

                        if (file_path.exists() && compare_files(&src_path, &file_path)) 
                            || (paths.len() > 1 && i < paths.len() - 1) {
                            println!("{} was copied to {}", 
                                     src_path.path_relative_from(&src_directory).unwrap().as_str().unwrap(), 
                                     new_path.path_relative_from(&dst_directory).unwrap().as_str().unwrap());
                        } else {
                            println!("{} was renamed to {}", 
                                     src_path.path_relative_from(&src_directory).unwrap().as_str().unwrap(), 
                                     new_path.path_relative_from(&dst_directory).unwrap().as_str().unwrap());
                        }

                    }
                }
            }
        }
    }
}
