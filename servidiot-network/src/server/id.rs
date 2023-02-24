use std::sync::atomic::{AtomicI32, Ordering};

/// An entity ID.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NetworkID(pub i32);

impl NetworkID {

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        static ID: AtomicI32 = AtomicI32::new(0);

        Self(ID.fetch_add(1, Ordering::SeqCst))
    }
}
