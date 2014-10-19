use std::io::stdio;
use std::io::process::Command;
use std::io::process::Process;
use std::io::IoError;
use std::collections::HashMap;
use std::str;
use std::vec::Vec;
use std::os::self_exe_name;
use std::str::CharSplits;
use util;

pub trait Server {
    fn get_metadata(&mut self, directory_path: &Path) -> HashMap<(u64, u64), Vec<String>>;
    fn get_checksum(&mut self, file_path: &Path) -> String;
    fn exit(&mut self);
}

pub struct StdServer {
    process: Process 
}

impl StdServer {
    pub fn new(host: String, ssh_port: int) -> StdServer {

        let mut process: Process;

        if host.as_slice() == "" {
            process = Command::new(self_exe_name().unwrap().as_str().unwrap()).arg("--server").spawn().unwrap();
        } else {
            process = Command::new("ssh").arg(format!("-p{}", ssh_port)).arg(host).arg("mvsync --server").spawn().unwrap();
        }

        StdServer { process: process }
    }

    fn read_until_the_end(&mut self) -> String
    {
        let mut output_pipe = self.process.stdout.as_mut().unwrap();
        let mut buffer: [u8, ..1024] = [0,..1024];
        let mut output: Vec<u8> = Vec::new();

        let mut readBytes;

        loop {
            readBytes = match output_pipe.read(buffer)
            {
                Ok(r) => r,
                Err(e) => fail!("{}", e)
            };

            output.push_all(buffer.slice(0, readBytes));

            if buffer.slice_to(readBytes-1).last() == Some(&0u8) {
                break;
            }
        }
        output.pop();

        return String::from_utf8(output).unwrap();
    }
}

impl Server for StdServer {

    fn get_checksum(&mut self, file_path: &Path) -> String
    {
        self.process.stdin.as_mut().unwrap().write_str("checksum ");
        self.process.stdin.as_mut().unwrap().write_line(file_path.as_str().unwrap());

        let checksum = self.read_until_the_end().replace("\n", "").replace("\0","");

        checksum
    }

    fn get_metadata(&mut self, directory_path: &Path) -> HashMap<(u64, u64), Vec<String>>
    {
        let mut map: HashMap<(u64, u64), Vec<String>> = HashMap::new();

        self.process.stdin.as_mut().unwrap().write_str("metadata ");
        self.process.stdin.as_mut().unwrap().write_line(directory_path.as_str().unwrap());

        let outputString = self.read_until_the_end();
        let mut lines = outputString.as_slice().split('\n');
        loop
        {
            let path = match lines.next() {
                Some(p) => from_str(p).unwrap(),
                None => break
            };
            let size = match lines.next() {
                Some(s) => from_str(s).unwrap(),
                None => break
            };
            let modified = match lines.next() {
                Some(m) => from_str(m).unwrap(),
                None => break
            };

            let key = (size, modified);
            if !map.contains_key(&key) {
                map.insert(key, Vec::new());
            }
            map.get_mut(&key).push(path);
        }

        map
    }

    fn exit(&mut self)
    {
        self.process.signal_kill().unwrap();
    }
}

pub struct InMemoryServer;

impl InMemoryServer {
    pub fn new() -> InMemoryServer {
        InMemoryServer
    }
}

impl Server for InMemoryServer {

    fn get_metadata(&mut self, directory_path: &Path) -> HashMap<(u64, u64), Vec<String>> {
        let mut metadata_map = HashMap::<(u64, u64), Vec<String>>::new();
        util::examine_files(directory_path, &mut metadata_map);
        return metadata_map;
    }

    fn get_checksum(&mut self, file_path: &Path) -> String {
        return util::get_file_checksum(file_path);
    }

    fn exit(&mut self) {
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
