mod primitives;
pub mod packet;
pub mod codec;
pub use primitives::*;
use std::io::Cursor;

/// Objects which can be read from byte arrays.
pub trait Readable: Sized {
    /// Read this object from a byte array.
    fn read_from(data: &mut Cursor<&[u8]>) -> anyhow::Result<Self>;
}

/// Objects which can be written to byte arrays.
pub trait Writable {
    /// Write this object to a byte array.
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()>;
}

/// Readable and writable.
pub trait Serializable: Readable + Writable {}
impl<T> Serializable for T where T: Readable + Writable {

}