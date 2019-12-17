//! The SharedQueueThreadPool is a simple version for rust-threadpool.

use super::ThreadPool;
use crate::error::Result;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Sentinel<'a> {
    pool: &'a Arc<SharedData>,
    active: bool,
}

impl<'a> Sentinel<'a> {
    pub fn new(pool: &Arc<SharedData>) -> Sentinel {
        Sentinel { pool, active: true }
    }

    pub fn cancel(&mut self) {
        self.active = false;
    }
}

impl<'a> Drop for Sentinel<'a> {
    fn drop(&mut self) {
        if self.active {
            if thread::panicking() {
                create_thread_in_pool(self.pool.clone())
            }
        }
    }
}

struct SharedData {
    receiver: Mutex<Receiver<Message>>,
}

enum Message {
    RunJob(Job),
    Shutdown,
}

pub struct SharedQueueThreadPool {
    sender: Sender<Message>,
    max_threads: usize,
    shared_data: Arc<SharedData>,
}

impl ThreadPool for SharedQueueThreadPool {
    fn new(max_threads: usize) -> Result<SharedQueueThreadPool> {
        // Create a channal, sender will send job from pool to sub thread.
        // Receiver will receive and execute job.
        let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();
        let receiver: Mutex<Receiver<Message>> = Mutex::new(receiver);
        let shared_data = Arc::new(SharedData { receiver });

        for _ in 0..max_threads {
            create_thread_in_pool(shared_data.clone());
        }

        Ok(SharedQueueThreadPool {
            sender,
            max_threads,
            shared_data,
        })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let msg: Message = Message::RunJob(Box::new(job));
        self.sender
            .send(msg)
            .expect("Fail to send task to sub thread.");
    }
}

// When thread pool is no-longer used, relative child threads should also be closed.
impl Drop for SharedQueueThreadPool {
    fn drop(&mut self) {
        for _ in 0..self.max_threads {
            self.sender.send(Message::Shutdown).unwrap_or(());
        }
    }
}

fn create_thread_in_pool(shared_data: Arc<SharedData>) {
    thread::spawn(move || {
        let mut sentinel: Sentinel = Sentinel::new(&shared_data);
        loop {
            let msg: Message = shared_data
                .receiver
                .lock()
                .expect("Can't acquire lock")
                .recv()
                .expect("Can't receive message from sender");

            match msg {
                Message::RunJob(job) => job(),
                Message::Shutdown => break,
            }
        }
        sentinel.cancel();
    });
}
