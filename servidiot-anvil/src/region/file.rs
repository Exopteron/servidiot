use std::{
    fs::File,
    io::{self, Cursor, Read, Seek, SeekFrom},
};

use bitvec::vec::BitVec;
use nbt::{from_gzip_reader, from_reader, from_zlib_reader};
use servidiot_primitives::position::ChunkPosition;
use thiserror::Error;

use super::nbt::ChunkRoot;

#[repr(u8)]
/// The compression type some chunk is stored in.
pub enum CompressionType {
    GZip = 1,
    ZLib = 2,
    Uncompressed = 3,
}
impl TryFrom<u8> for CompressionType {
    type Error = UnknownCompressionType;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::GZip),
            2 => Ok(Self::ZLib),
            3 => Ok(Self::Uncompressed),
            n => Err(UnknownCompressionType(n)),
        }
    }
}

#[derive(Error, Debug)]
#[error("Unknown compression type {0}")]
pub struct UnknownCompressionType(pub u8);

/// A loaded region file on-disk.
pub struct RegionFile {
    /// Location table showing where each
    /// chunk is located within the file.
    chunk_location: [u8; 4096],
    /// The last modification time of
    /// each chunk.
    timestamps: [u32; 1024],
    /// Which sectors are free within
    /// the region file.
    free_sectors: BitVec,
    /// The file handle.
    file: File,
}

#[derive(Error, Debug)]
pub enum ChunkError {
    /// Reported when a chunk is not
    /// present within the region file.
    #[error("chunk {0}: not present in region file")]
    ChunkNotPresent(ChunkPosition),
    /// Reported when an unknown
    /// compression type is found.
    #[error("chunk {0}: {1}")]
    UnknownCompressionType(ChunkPosition, UnknownCompressionType),
    /// Reported when an NBT error
    /// comes up.
    #[error("chunk {0}: {1}")]
    NBTError(ChunkPosition, nbt::Error),
    /// Reported when an IO error
    /// comes up.
    #[error("chunk {0}: {1}")]
    IOError(ChunkPosition, io::Error),
}

type ChunkResult<T> = std::result::Result<T, ChunkError>;

impl RegionFile {
    pub const BYTES_PER_SECTOR: u64 = 4096;

    /// Loads a file as a region file.
    pub fn new(mut file: File) -> io::Result<Self> {
        file.rewind()?;
        let mut chunk_location = [0; 4096];
        file.read_exact(&mut chunk_location)?;
        let mut timestamps = [0; 4096];
        file.read_exact(&mut timestamps)?;

        let mut this = Self {
            chunk_location,
            timestamps: unsafe { std::mem::transmute(timestamps) },
            free_sectors: BitVec::new(),
            file,
        };
        let file_len = this.file.metadata()?.len();

        for _ in 0..(file_len / Self::BYTES_PER_SECTOR) {
            this.free_sectors.push(true);
        }

        this.free_sectors.set(0, false); // chunk locations
        this.free_sectors.set(1, false); // timestamps

        let mut i = 0;
        while i < this.chunk_location.len() {
            let v = &this.chunk_location[i..i + 4];
            let offset = u32::from_be_bytes([0, v[0], v[1], v[2]]) as usize;
            let size = v[3] as usize;
            for v in offset..offset + size {
                this.free_sectors.set(v, false);
            }
            i += 4;
        }

        Ok(this)
    }

    /// Gets the on-disk location of
    /// some chunk.
    fn get_chunk_location(&self, position: ChunkPosition) -> Option<(u32, u8)> {
        let offset = 4 * ((position.x & 31) + (position.z & 31) * 32);
        let v = &self.chunk_location[offset as usize..(offset + 4) as usize];
        let offset = u32::from_be_bytes([0, v[0], v[1], v[2]]);
        if offset == 0 && v[3] == 0 {
            None
        } else {
            Some((offset, v[3]))
        }
    }

    /// Gets the on-disk timestamp
    /// of some chunk.
    fn get_chunk_timestamp(&self, position: ChunkPosition) -> u32 {
        let offset = ((position.x & 31) + (position.z & 31) * 32) as usize;
        self.timestamps[offset]
    }

    /// Reads a chunk from this region file.
    ///
    /// Returns the chunk data and
    /// last-modified timestamp.
    pub fn read_chunk(&mut self, chunk_position: ChunkPosition) -> ChunkResult<(ChunkRoot, u32)> {
        let timestamp = self.get_chunk_timestamp(chunk_position);
        let (offset, _) = self
            .get_chunk_location(chunk_position)
            .ok_or(ChunkError::ChunkNotPresent(chunk_position))?;
        let position = (offset as u64) * 4096;

        self.file
            .seek(SeekFrom::Start(position))
            .map_err(|v| ChunkError::IOError(chunk_position, v))?;
        let mut byte_size = [0; 4];
        self.file
            .read_exact(&mut byte_size)
            .map_err(|v| ChunkError::IOError(chunk_position, v))?;
        let exact_size = u32::from_be_bytes(byte_size) as usize;

        let mut compression_type = [0; 1];
        self.file
            .read_exact(&mut compression_type)
            .map_err(|v| ChunkError::IOError(chunk_position, v))?;
        let compression_type = CompressionType::try_from(compression_type[0])
            .map_err(|v| ChunkError::UnknownCompressionType(chunk_position, v))?;

        let mut data = vec![0; exact_size];
        self.file
            .read_exact(&mut data)
            .map_err(|v| ChunkError::IOError(chunk_position, v))?;

        let data = Cursor::new(data);

        let data = match compression_type {
            CompressionType::GZip => from_gzip_reader(data),
            CompressionType::ZLib => from_zlib_reader(data),
            CompressionType::Uncompressed => from_reader(data),
        }
        .map_err(|v| ChunkError::NBTError(chunk_position, v))?;

        Ok((data, timestamp))
    }
}
