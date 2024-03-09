#![allow(unused)]

use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self, Thread},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
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

            Ok(ThreadPool {
                workers,
                sender: Some(sender),
            })
        } else {
            Err(PoolCreationError::new("Pool size must be greater than 0"))
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // let thread = thread::spawn(move || loop {
        //     let job = receiver
        //         .lock()
        //         .expect("Failed to lock receiver")
        //         .recv()
        //         .expect("Failed to receive job");

        //     println!("Worker {} got a job; executing.", id);
        //     job();
        // });
        // Worker {
        //     id,
        //     thread: Some(thread),
        // }
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().expect("Failed to lock receiver").recv();

            match message {
                Ok(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Err(_) => {
                    println!("Worker {} failed to receive job", id);
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
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
