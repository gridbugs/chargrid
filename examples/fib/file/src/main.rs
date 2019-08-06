extern crate fib;
extern crate prototty_file_storage;

use fib::App;
use prototty_file_storage::{FileStorage, IfDirectoryMissing};

fn main() {
    let storage = FileStorage::next_to_exe("storage", IfDirectoryMissing::Create).unwrap();
    let mut app = App::new(storage);
    println!("{}", app.get());
    app.next_and_save();
}
