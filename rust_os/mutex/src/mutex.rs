#![allow(unused)]

use std::sync::{Arc, Mutex};
use std::thread;

pub fn main() {
    // Atomic Reference Counting (Arc) is a thread-safe reference-counting pointer
    // that is used to share ownership between threads.
    let counter = Arc::new(Mutex::new(0));
    let mut handles = Vec::with_capacity(10);

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();

            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result: {}", *counter.lock().unwrap());
    
}

// Mutex can cause deadlocks if not used properly.
// Deadlocks occur when two or more threads wait for each other
// to release the lock, causing the program to hang indefinitely.
// To avoid deadlocks, always acquire locks in the same order.
// For example, if you have two mutexes,
// always acquire the first mutex before the second mutex.
// Also, always release the locks in the reverse order of acquisition.
// This way, you can avoid deadlocks.
