use std::ops::Deref;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
/// A block ID. Goes from
/// `0` to `4095`.
pub struct BlockID(u16);
impl Deref for BlockID {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl BlockID {

    /// Creates a new `BlockID`.
    /// If `block_id` is greater than 4095, 
    /// returns `None`.
    pub const fn new(block_id: u16) -> Option<Self> {
        if block_id > 4095 {
            None
        } else {
            Some(Self(block_id))
        }
    }

    /// # Safety
    /// Ensure `block_id` is no larger than 4096.
    pub const unsafe fn new_unchecked(block_id: u16) -> Self {
        Self(block_id)
    }
}


pub trait BlockType {
    const VALID_BLOCK_TYPES: &'static [BlockID];
}
