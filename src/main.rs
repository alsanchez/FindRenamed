use std::io::fs::rename;
use std::io::fs::PathExtensions;
use std::collections::HashMap;
use std::os;
use std::vec::Vec;

mod util;
mod server;

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

    let server = server::InMemoryServer::new();
    let src_directory = Path::new(args[1].clone());
    let dst_directory = Path::new(args[2].clone());
    sync_renames(&server, &src_directory, &dst_directory);
}

fn sync_renames(server: &server::Server, src_directory: &Path, dst_directory: &Path) {

    let src_map = server.get_metadata(src_directory);
    let new_map = server.get_metadata(dst_directory);
    for (&(size, mod_date), paths) in new_map.iter() {

        let mut matching_paths = 0i;

        for (i, value) in paths.iter().enumerate() {

            let new_path = Path::new(value.clone());
            let src_path :Path; 

                
            match find_matching_file(&new_path, size, mod_date, &src_map, server) {
                Some(p) => src_path = p,
                None => continue
            }

            let mut file_path = dst_directory.clone();
            file_path.push(src_path.path_relative_from(src_directory).unwrap());

            let src_relative_path = src_path.path_relative_from(src_directory).unwrap();
            let dst_relative_path = new_path.path_relative_from(dst_directory).unwrap();

            // No point in comparing a file with itself
            if src_relative_path == dst_relative_path {
                continue;
            }

            matching_paths += 1;

            if matching_paths > 1 && file_path.exists() && server.get_checksum(&src_path) == server.get_checksum(&file_path) {
                println!("{} was copied to {}", src_relative_path.display(), dst_relative_path.display()); 

            } else {
                println!("{} was renamed to {}", src_relative_path.display(), dst_relative_path.display());
                rename(
                    &src_directory.join(src_relative_path),
                    &src_directory.join(dst_relative_path)).unwrap();
            }
        }
    }
}

fn find_matching_file(file: &Path, size: u64, modification_date: u64, master_metadata: &HashMap<(u64, u64), Vec<String>>, server: &server::Server) -> Option<Path> {

    // Check whether there are any files in master with the same
    // size and modification date of the sought file
    match master_metadata.find(&(size, modification_date)) {

        Some(matches) => {

            // Compare the file contents until we find a match, or return None if
            // we don't find one
            for potential_match in matches.iter() {
                let path = Path::new(potential_match.clone());
                if server.get_checksum(&path) == server.get_checksum(file) {
                    return Some(path);
                }
            }

            return None

        }

        None => return None
    }

}

