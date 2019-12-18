mod naive;
mod rayon;
mod shared_queue;
use crate::error::Result;

pub use naive::NaiveThreadPool;
pub use self::rayon::RayonThreadPool;
pub use shared_queue::SharedQueueThreadPool;

pub trait ThreadPool {
    fn new(max_threads: usize) -> Result<Self>
    where
        Self: Sized;

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}
