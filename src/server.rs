use std::io::stdio;
use std::io::IoError;
use std::collections::HashMap;
use std::str;
use std::vec::Vec;
use util;

pub trait Server {
    fn get_metadata(&self, directory_path: &Path) -> HashMap<(u64, u64), Vec<String>>;
    fn get_checksum(&self, file_path: &Path) -> String;
    fn exit(&self);
}

pub struct InMemoryServer;

impl InMemoryServer {
    pub fn new() -> InMemoryServer {
        InMemoryServer
    }
}

impl Server for InMemoryServer {

    fn get_metadata(&self, directory_path: &Path) -> HashMap<(u64, u64), Vec<String>> {
        let mut metadata_map = HashMap::<(u64, u64), Vec<String>>::new();
        util::examine_files(directory_path, &mut metadata_map);
        return metadata_map;
    }

    fn get_checksum(&self, file_path: &Path) -> String {
        return util::get_file_checksum(file_path);
    }

    fn exit(&self) {
        // No-op
    }
}

pub fn start_server() {

    match manage_client(&mut stdio::stdin_raw(), &mut stdio::stdout_raw()) {
        Ok(_) => {}
        Err(e) => println!("{}", e)
    }
}

fn manage_client(input: &mut Reader, output: &mut Writer) -> Result<(), IoError> {

    let mut buffer = [0u8, ..4096];

    loop {
        let read_bytes = input.read(buffer).unwrap();
        let command = str::from_utf8(buffer.slice(0, read_bytes)).unwrap().trim();

        if command.starts_with("checksum") {
            let file_path = command.replace("checksum ", "");

            // Compute the checksum
            let checksum = util::get_file_checksum(&Path::new(file_path));

            // Compose the output
            let output_string = format!("OK {}\n", checksum);

            // Return a response
            try!(output.write(output_string.as_bytes()));

        } else if command.starts_with("metadata") {
            let dir_path = command.replace("metadata ", "");
            let mut file_map = HashMap::<(u64, u64), Vec<String>>::new();
            util::examine_files(&Path::new(dir_path), &mut file_map);
            try!(output.write(b"OK START\n"));
            for (key, values) in file_map.iter() {
                for value in values.iter() {
                    let &(size, mod_date) = key;
                    let line = format!("{} {} {}{}\n", size, mod_date, value, 0u8);
                    try!(output.write(line.as_bytes()));
                }
            }
            try!(output.write(b"END\n"));
        }
        else if command == "exit" {
            break;
        } else {

            // Compose the output
            let output_string = format!("ERR Invalid command {}\n", command);

            // And return the response
            try!(output.write(output_string.as_bytes()));
        }
    }

    Ok(())

}
