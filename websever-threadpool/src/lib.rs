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
        // Arc: enable multiple owner
        // Mutex: only 1 worker gets the job at a time
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
        // FnOnce: xecute a request once;
        // Send: transfer the closure from 1 thread to another;
        // 'static: don't know
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap(); // send the job
    }

    fn drop(&mut self) {
        for worker in &mut self.workers {
            println!("shuting down worker id {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                // take the value and replace it with None
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>, // to replace some with none when drop
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            // let job = receiver.lock().unwrap().recv().unwrap(); // block for the next job

            // println!("Worker {id} is executing");

            // job(); // do the job

            let msg = receiver.lock().unwrap().recv();

            match msg {
                Ok(job) => {
                    println!("worker {id} is executing");

                    job()
                }
                Err(_) => {
                    println!("worker {id} is disconnected");

                    break; // stop the loop
                }
            }
        });

        Worker {
            id: id,
            thread: Some(thread),
        }
    }
}
