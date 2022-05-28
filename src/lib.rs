use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub enum ThreadPoolStatus {
    Action,
    Terminate,
}

enum Message {
    NewJob(Job, Cb),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

type Job = Box<dyn FnOnce() -> ThreadPoolStatus + Send + 'static>;
type Cb = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    ///Create new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F1, F2>(&self, f: F1, callback: F2)
    where
        F1: FnOnce() -> ThreadPoolStatus,
        F1: Send + 'static,
        F2: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender
            .send(Message::NewJob(job, Box::new(callback)))
            .unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending termination messages to workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers");

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
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let msg = receiver.lock().unwrap().recv().unwrap();

            match msg {
                Message::NewJob(job, callback) => {
                    println!("Worker {} got a job; executing.", id);

                    let res: ThreadPoolStatus = job();

                    if let ThreadPoolStatus::Terminate = res {
                        callback();
                    }
                }
                Message::Terminate => {
                    println!("Terminating worker {}.", id);

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
