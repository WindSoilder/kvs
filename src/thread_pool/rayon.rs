use super::ThreadPool;
use crate::error::Result;

pub struct RayonThreadPool;

impl ThreadPool for RayonThreadPool {
    fn new(max_threads: usize) -> Result<RayonThreadPool> {
        Ok(RayonThreadPool)
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
    }
}
