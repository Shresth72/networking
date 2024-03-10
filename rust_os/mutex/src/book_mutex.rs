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

    // *my_book.name.lock().unwrap() = "Rust Programming 101";

    let mut mutex_changer = my_book.author.lock();

    // Drop the mutex_changer
    std::mem::drop(mutex_changer);

    // The data will unreachable if lock is already acquired
    if let Ok(mut mutex) = my_book.author.try_lock() {
        *mutex = "Harry Potter";
    }

    println!("{:?}", my_book);
}
