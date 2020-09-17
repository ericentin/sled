use std::panic::UnwindSafe;

use rayon_core::{ThreadPool, ThreadPoolBuilder};
use rustler::JobSpawner;

lazy_static! {
    static ref POOL: ThreadPool = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get() * 2) // IO-bound workload
        .build()
        .unwrap();
}
pub struct ThreadPoolSpawner;

impl JobSpawner for ThreadPoolSpawner {
    fn spawn<F: FnOnce() + Send + UnwindSafe + 'static>(job: F) {
        POOL.spawn(job)
    }
}
