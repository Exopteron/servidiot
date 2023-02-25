use std::sync::atomic::{AtomicI32, Ordering};

use crate::io::{Writable, Readable};

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

impl Writable for NetworkID {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        self.0.write_to(target)
    }
}
impl Readable for NetworkID {
    fn read_from(data: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<Self> {
        Ok(Self(i32::read_from(data)?))
    }
}