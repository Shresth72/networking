#![allow(unused)]

use std::sync::Mutex;

#[derive(Debug)]
struct Book<'a> {
    name: Mutex<&'a str>,
    author: Mutex<&'a str>,
    sold: Mutex<u32>,
}

pub fn main() {
    let my_book = Book {
        name: Mutex::new("Rust Programming"),
        author: Mutex::new("John Doe"),
        sold: Mutex::new(100),
    };

    *my_book.name.lock().unwrap() = "Rust Programming 101";

    println!("{:?}", my_book);
}
