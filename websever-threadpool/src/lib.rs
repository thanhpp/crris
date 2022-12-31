/*
Created a limited number of thread to handle requests
*/

use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size); // pre-allocate

        for id in 0..size {
            // create threads and stores them in the vec
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers: workers,
            sender: sender,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static, // eFnOnce: xecute a request once; Send: transfer the closure from 1 thread to another; 'static: don't know
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
            let job = receiver.lock().unwrap().recv().unwrap();

            println!("Worker {id} is executing");

            job();
        });

        Worker {
            id: id,
            thread: thread,
        }
    }
}
