use crossbeam::utils::CachePadded;
use std::ops::Deref;
use std::sync::atomic;

pub struct WorkerSync(Vec<CachePadded<atomic::AtomicBool>>);

impl WorkerSync {
    pub fn new(size: usize) -> Self {
        let mut new_sync = Vec::new();
        for _ in 0..size {
            new_sync.push(CachePadded::new(atomic::AtomicBool::new(false)));
        }
        return WorkerSync(new_sync);
    }

    pub fn all(&self) -> bool {
        for worker_idx in 0..self.len() {
            if self[worker_idx].load(atomic::Ordering::Relaxed) {
                return true;
            }
        }
        return false;
    }
}

impl Deref for WorkerSync {
    type Target = Vec<CachePadded<atomic::AtomicBool>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
