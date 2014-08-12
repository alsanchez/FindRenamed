extern crate rust_crypto = "rust-crypto";

use rust_crypto::digest::Digest;
use rust_crypto::sha1::Sha1;

fn main() {
    let mut hasher = Sha1::new();
    hasher.input_str("test");
    println!("{}\r\n", hasher.result_str());
}
