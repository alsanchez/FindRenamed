use std::io::{Listener, Acceptor};
use std::io::net::tcp::TcpListener;

fn main() {
    let mut acceptor = TcpListener::bind("127.0.0.1", 9876).listen().unwrap();
    println!("listening started, ready to accept");
    for opt_stream in acceptor.incoming() {
        spawn(proc() {
            let mut stream = opt_stream.unwrap();
            let byte = stream.read_byte().unwrap();
            stream.write([byte]).unwrap();
            stream.write(b"\r\n").unwrap();
        });
    }
}
