//! The entity metadata format.
//! 
//! https://wiki.vg/index.php?title=Entity_metadata&oldid=5909#Entity_Metadata

use ahash::HashMap;
use thiserror::Error;

use crate::position::BlockPosition;

/// The metadata store.
#[derive(Default, Debug)]
pub struct Metadata {
    values: HashMap<u8, MetadataItem>,
    synced: bool
}
pub type MetadataResult<T> = std::result::Result<T, MetadataError>;

impl Metadata {


    pub fn insert(&mut self, id: u8, value: MetadataItem) {
        assert!(id <= 0x1F, "Metadata keys are 5 bits in length");
        self.synced = false;
        self.values.insert(id, value);
    }

    pub fn fetch(&self, id: u8) -> MetadataResult<&MetadataItem> {
        if let Some(v) = self.values.get(&id) {
            Ok(v)
        } else {
            Err(MetadataError::NotPresent(id))
        }
    }

    pub fn values(&self) -> impl Iterator<Item = (u8, &MetadataItem)> {
        self.values.keys().copied().zip(self.values.values())
    }

    pub fn is_synced(&self) -> bool {
        self.synced
    }

    pub fn set_synced(&mut self) {
        self.synced = true;
    }
}

#[repr(u8)]
pub enum MetadataTypeKey {
    Byte = 0,
    Short = 1,
    Int = 2,
    Float = 3,
    String = 4,
    Slot = 5,
    Position = 6
}

/// An item present in the metadata.
#[derive(Debug)]
pub enum MetadataItem {
    Byte(u8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    Slot, // TODO
    Position(BlockPosition)
}

impl MetadataItem {
    pub fn type_key(&self) -> MetadataTypeKey {
        match self {
            MetadataItem::Byte(_) => MetadataTypeKey::Byte, 
            MetadataItem::Short(_) => MetadataTypeKey::Short,
            MetadataItem::Int(_) => MetadataTypeKey::Int,
            MetadataItem::Float(_) => MetadataTypeKey::Float,
            MetadataItem::String(_) => MetadataTypeKey::String,
            MetadataItem::Slot => MetadataTypeKey::Slot,
            MetadataItem::Position(_) => MetadataTypeKey::Position,
        }
    }
}

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("metadata entry {0} not present")]
    NotPresent(u8),
    #[error("metadata entry {0} not of expected type")]
    WrongType(u8)
}