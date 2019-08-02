extern crate fib;
extern crate prototty_monolithic_storage;

use fib::App;
use prototty_monolithic_storage::*;
use std::fs::File;
use std::io::{Read, Write};

struct TmpFile;

const PATH: &'static str = "/tmp/foo";

impl StoreBytes for TmpFile {
    fn store(&mut self, bytes: &[u8]) {
        let mut f = File::create(PATH).unwrap();
        f.write_all(bytes).unwrap();
    }
}

fn main() {
    let storage = if let Ok(mut file) = File::open(PATH) {
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        MonoStorage::from_bytes_or_empty(&buf, TmpFile)
    } else {
        MonoStorage::new(TmpFile)
    };
    let mut app = App::new(storage);
    println!("{}", app.get());
    app.next_and_save();
}
