use super::ThreadPool;
use crate::error::Result;
use std::thread;

pub struct NaiveThreadPool;

impl ThreadPool for NaiveThreadPool {
    fn new(_max_threads: usize) -> Result<NaiveThreadPool> {
        Ok(NaiveThreadPool)
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(job);
    }
}
