use std::sync::atomic::{AtomicU16, Ordering};

#[derive(Debug, PartialEq, Eq)]
pub struct UniqueId (u16);
impl UniqueId {
    pub fn new() -> Self {
        static COUNTER: AtomicU16 = AtomicU16::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst); 
        return UniqueId(id);
    }

}