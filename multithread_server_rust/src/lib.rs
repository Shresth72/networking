#![allow(unused)]

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size > 0 {
            let (sender, receiver) = mpsc::channel();
            let receiver = Arc::new(Mutex::new(receiver));

            let mut workers = Vec::with_capacity(size);
            for id in 0..size {
                workers.push(Worker::new(id, Arc::clone(&receiver)));
            }

            Ok(ThreadPool { workers, sender })
        } else {
            Err(PoolCreationError::new("Pool size must be greater than 0"))
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver
                .lock()
                .expect("Failed to lock receiver")
                .recv()
                .expect("Failed to receive job");

            println!("Worker {} got a job; executing.", id);
            job();
        });
        Worker { id, thread }
    }
}

#[derive(Debug)]
pub struct PoolCreationError {
    pub details: String,
}

impl PoolCreationError {
    pub fn new(msg: &str) -> PoolCreationError {
        PoolCreationError {
            details: msg.to_string(),
        }
    }
}
