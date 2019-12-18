use super::ThreadPool;
use crate::error::Result;
use rayon::ThreadPoolBuilder;

pub struct RayonThreadPool {
    inner: rayon::ThreadPool,
}

impl ThreadPool for RayonThreadPool {
    fn new(max_threads: usize) -> Result<RayonThreadPool> {
        Ok(RayonThreadPool {
            inner: ThreadPoolBuilder::new()
                .num_threads(max_threads)
                .build()
                .expect("Create rayon thread pool failed"),
        })
    }

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.inner.spawn(job);
    }
}
