use std::io::stdio;
use std::collections::HashMap;
use std::str;
use std::vec::Vec;
use util;

pub fn start_server() {

    manage_client(&mut stdio::stdin_raw(), &mut stdio::stdout_raw());
}

fn manage_client(input: &mut Reader, output: &mut Writer) {

    let mut buffer = [0u8, ..4096];

    loop {
        let read_bytes = input.read(buffer).unwrap();
        let command = str::from_utf8(buffer.slice(0, read_bytes)).unwrap().trim();

        if command.starts_with("checksum") {
            let file_path = command.replace("checksum ", "");
            output.write(b"OK ");
            output.write(util::get_file_checksum(&Path::new(file_path)).as_bytes());
            output.write(b"\n");
        } else if command.starts_with("metadata") {
            let dir_path = command.replace("metadata ", "");
            let mut file_map = HashMap::<(u64, u64), Vec<String>>::new();
            util::examine_files(&Path::new(dir_path), &mut file_map);
            output.write(b"OK START\n");
            for (key, values) in file_map.iter() {
                for value in values.iter() {
                    let &(size, mod_date) = key;
                    output.write(size.to_string().as_bytes());
                    output.write(b" ");
                    output.write(mod_date.to_string().as_bytes());
                    output.write(b" ");
                    output.write(value.as_bytes());
                    output.write_u8(0u8);
                    output.write(b"\n");
                }
            }
            output.write(b"END\n");
        }
        else if command == "exit" {
            break;
        } else {
            output.write(b"ERR Invalid command \"");
            output.write(command.as_bytes());
            output.write(b"\"\n");
        }
    }

}
