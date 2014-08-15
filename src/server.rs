use std::io::{Listener, Acceptor};
use std::collections::HashMap;
use std::str;
use std::vec::Vec;
use std::io::net::tcp::TcpListener;
use util;

pub fn start_server() {

    let mut acceptor = TcpListener::bind("127.0.0.1", 9876).listen().unwrap();
    println!("listening started, ready to accept");
    let opt_stream = acceptor.accept();
    let mut buffer = [0u8, ..4096];
    let mut stream = opt_stream.unwrap();
    loop {
        let read_bytes = stream.read(buffer).unwrap();
        let command = str::from_utf8(buffer.slice(0, read_bytes)).unwrap().trim();

        if command.starts_with("checksum") {
            let file_path = command.replace("checksum ", "");
            stream.write(b"OK ");
            stream.write(util::get_file_checksum(&Path::new(file_path)).as_bytes());
            stream.write(b"\n");
        } else if command.starts_with("metadata") {
            let dir_path = command.replace("metadata ", "");
            let mut file_map = HashMap::<(u64, u64), Vec<String>>::new();
            util::examine_files(&Path::new(dir_path), &mut file_map);
            for (key, values) in file_map.iter() {
                for value in values.iter() {
                    let &(size, mod_date) = key;
                    stream.write(size.to_string().as_bytes());
                    stream.write(b" ");
                    stream.write(mod_date.to_string().as_bytes());
                    stream.write(b" ");
                    stream.write(value.as_bytes());
                    stream.write_u8(0u8);
                    stream.write(b"\n");
                }
            }
        }
        else if command == "exit" {
            break;
        } else {
            stream.write(b"ERR Invalid command \"");
            stream.write(command.as_bytes());
            stream.write(b"\"\n");
        }
    }
}

