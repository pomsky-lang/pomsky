use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct Count {
    count: AtomicUsize,
}

impl Count {
    pub(super) fn add_one(&self) {
        self.count.fetch_add(1, Ordering::AcqRel);
    }

    pub fn get_count(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}
