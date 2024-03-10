#![allow(unused)]

mod mutex;
use mutex::main as mutex_main;

mod book_mutex;
use book_mutex::main as book_mutex_main;

fn main() {
    mutex_main();

    // book_mutex_main();
}
