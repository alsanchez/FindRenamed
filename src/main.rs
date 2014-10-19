use std::io::fs::rename;
use std::io::fs::PathExtensions;
use std::collections::HashMap;
use std::os;
use std::io;
use std::vec::Vec;
use std::str::StrSlice;

mod util;
mod server;

fn main() {

    let args = os::args();

    if args.len() == 2 && args[1] == "--server".to_string() {
        serve(); 
        return;
    }

    if args.len() == 2 && args[1] == "--start-server".to_string() {
        server::start_server();
        return;
    }

    let use_external_process = args.contains(&String::from_str("--use-external-process"));

    if args.len() < 3 {
        println!("Usage: {} <original-directory> <new-directory>", args[0]);
        return;
    }

    let remote_source = args[1].as_slice().contains(":");
    let dry_run = args.contains(&"--dry-run".to_string());
    let mut ssh_port = 22i;
    let host: String;
    let source_path: String;
    let verbose = args.contains(&"--verbose".to_string());
    let checksums = !args.contains(&"--no-checksums".to_string());

    if args.contains(&"--ssh-port".to_string()) {
        let index = args.iter().position(|a| a == &"--ssh-port".to_string()).unwrap();
        ssh_port = from_str(args.get(index + 1).as_slice()).unwrap();
    }

    if remote_source {
        let components: Vec<&str> = args[1].as_slice().split(':').collect();
        host = components[0].to_string();
        source_path = components[1].to_string();
    }
    else {
        source_path = args[1].clone();
        host = "".to_string();
    }

    let src_directory = Path::new(source_path);
    let dst_directory = Path::new(args[2].clone());

    if remote_source || use_external_process {
        let mut server = server::StdServer::new(host, ssh_port);
        let mut dest_server = server::InMemoryServer::new();
        sync_renames(checksums, verbose, dry_run, &mut server, &mut dest_server, &src_directory, &dst_directory);
    }
    else {
        let mut server = server::InMemoryServer::new();
        let mut dest_server = server::InMemoryServer::new();
        sync_renames(checksums, verbose, dry_run, &mut server, &mut dest_server, &src_directory, &dst_directory);
    }
}

fn sync_renames(checksums:bool, verbose: bool, dry_run: bool, source_server: &mut server::Server, dest_server: &mut server::Server, src_directory: &Path, dst_directory: &Path) {

    let mut renames: Vec<(Path, Path)> = Vec::new();

    if verbose {
        println!("Getting source metadata...");
    }
    let src_map = source_server.get_metadata(src_directory);
    if verbose {
        println!("Getting destination metadata...");
    }
    let new_map = dest_server.get_metadata(dst_directory);

    for (&(size, mod_date), paths) in new_map.iter() {

        let mut matching_paths = 0i;

        for (i, value) in paths.iter().enumerate() {

            let new_path = Path::new(value.clone());
            let src_path :Path;

            match find_matching_file(src_directory, dst_directory, checksums, verbose, &new_path, size, mod_date, &src_map, source_server, dest_server) {
                Some(p) => src_path = p,
                None => continue
            }

            let mut file_path = dst_directory.clone();
            file_path.push(src_path.path_relative_from(src_directory).unwrap());

            let src_relative_path = src_path.path_relative_from(src_directory).unwrap();
            let dst_relative_path = new_path.path_relative_from(dst_directory).unwrap();

            matching_paths += 1;

            if matching_paths > 1 && file_path.exists() && source_server.get_checksum(&src_path) == dest_server.get_checksum(&file_path) {
                println!("{} was copied to {}", dst_relative_path.display(), src_relative_path.display()); 

            } else {
                println!("{} was renamed to {}", dst_relative_path.display(), src_relative_path.display());
                renames.push(
                    (dst_directory.join(dst_relative_path),
                    dst_directory.join(src_relative_path)));
            }
        }
    }

    if !dry_run {
        for &(ref old, ref new) in renames.iter() {
            rename(old, new).unwrap();
        }
    }
}

fn find_matching_file(src_directory: &Path, dst_directory: &Path, checksums: bool, verbose: bool, file: &Path, size: u64, modification_date: u64, master_metadata: &HashMap<(u64, u64), Vec<String>>, src_server: &mut server::Server, dest_server: &mut server::Server) -> Option<Path> {

    // Check whether there are any files in master with the same
    // size and modification date of the sought file
    match master_metadata.find(&(size, modification_date)) {

        Some(matches) => {

            // Compare the file contents until we find a match, or return None if
            // we don't find one
            for potential_match in matches.iter() {

                let path = Path::new(potential_match.clone());

                let src_relative_path = path.path_relative_from(src_directory).unwrap();
                let dst_relative_path = file.path_relative_from(dst_directory).unwrap();

                // No point in comparing a file with itself
                if src_relative_path == dst_relative_path {
                    continue;
                }

                if !checksums {
                    return Some(path);
                }

                if verbose {
                    println!("Getting checksum for source file '{}'...", path.as_str());
                }
                let src_checksum = src_server.get_checksum(&path);
                if verbose {
                    println!("Getting checksum for destination file '{}'...", file.as_str());
                }
                let dest_checksum = dest_server.get_checksum(file);
                if src_checksum == dest_checksum {
                    return Some(path);
                }
            }

            return None

        }

        None => return None
    }

}

fn serve() {

    for line in io::stdin().lines() {
        let unwrapped_line = line.unwrap();
        let line_slice = unwrapped_line.as_slice().trim_right_chars('\n');
        if line_slice.starts_with("checksum ") {
           let file_name = line_slice.slice_from(9); 
           println!("{}", util::get_file_checksum(&Path::new(file_name)));
           println!("\0");
        }

        if line_slice.starts_with("metadata ") {
            let path = line_slice.slice_from(9);
            for metadata in util::MetadataReader::new(&Path::new(path)) { 
                println!("{}", metadata.path);
                println!("{}", metadata.size);
                println!("{}", metadata.modified);
            }
            println!("\0");

        }

        
    }
}
