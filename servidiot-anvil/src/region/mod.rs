use std::{
    fs::File,
    io,
    path::PathBuf,
    time::{Duration, SystemTime, SystemTimeError, UNIX_EPOCH},
};

use ahash::AHashMap;
use servidiot_primitives::position::{ChunkPosition, RegionPosition};
use thiserror::Error;

use self::{
    file::{ChunkError, CompressionType, RegionFile},
    nbt::ChunkRoot,
};

pub mod file;
pub mod nbt;

/// Manages regions within a directory.
pub struct RegionManager {
    /// The directory containing the regions.
    directory: PathBuf,
    /// Loaded region cache.
    cache: AHashMap<RegionPosition, RegionFile>,
    /// The compression method to use when saving.
    compression_method: CompressionType,
}

#[derive(Error, Debug)]
pub enum RegionManagerError {
    #[error("IO error: {0}")]
    IOError(io::Error),
    #[error("Chunk error: {0}")]
    ChunkError(ChunkError),
    #[error("Time error: {0}")]
    SystemTimeError(SystemTimeError),
}

impl From<io::Error> for RegionManagerError {
    fn from(value: io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<ChunkError> for RegionManagerError {
    fn from(value: ChunkError) -> Self {
        Self::ChunkError(value)
    }
}

pub type RegionManagerResult<T> = std::result::Result<T, RegionManagerError>;

impl RegionManager {
    /// Creates a new region manager
    /// for the directory `directory`.
    pub fn new(directory: PathBuf, compression_method: CompressionType) -> Self {
        Self {
            compression_method,
            directory,
            cache: AHashMap::default(),
        }
    }

    /// Loads chunk data. Returns data and timestamp.
    pub fn load_chunk(
        &mut self,
        position: ChunkPosition,
    ) -> RegionManagerResult<(ChunkRoot, SystemTime)> {
        let (region, _) = self.load_region(position.region())?;
        let (data, time) = region.read_chunk(position)?;

        Ok((data, UNIX_EPOCH + Duration::from_secs(time as u64)))
    }

    /// Saves chunk data.
    pub fn save_chunk(
        &mut self,
        position: ChunkPosition,
        data: ChunkRoot,
    ) -> RegionManagerResult<()> {
        let compression = self.compression_method;
        let (region, _) = self.load_region(position.region())?;
        region
            .write_chunk(
                compression,
                position,
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(RegionManagerError::SystemTimeError)?
                    .as_secs() as u32,
                data,
            )
            .map_err(RegionManagerError::ChunkError)
    }

    /// Flush the whole cache.
    pub fn flush_cache(&mut self) -> RegionManagerResult<()> {
        for (_, data) in &mut self.cache {
            data.flush()?;
        }
        Ok(())
    }

    /// Removes a region from cache. Returns
    /// `true` if the region was present in cache.
    pub fn unload_region(&mut self, position: RegionPosition) -> RegionManagerResult<bool> {
        if let Some(mut v) = self.cache.remove(&position) {
            v.flush()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Caches a region. Returns `true` if
    /// the region was already cached.
    pub fn load_region(
        &mut self,
        position: RegionPosition,
    ) -> RegionManagerResult<(&mut RegionFile, bool)> {
        if self.cache.contains_key(&position) {
            return Ok((self.cache.get_mut(&position).unwrap(), true));
        } else {
            let mut path = self.directory.clone();
            path.push(format!("r.{}.{}.mca", position.x, position.z));

            let mut opts = File::options();
            opts.read(true).write(true);

            let f = if !path.try_exists()? {
                opts.create(true);
                RegionFile::create
            } else {
                RegionFile::open
            };

            let file = f(opts.open(path)?)?;
            self.cache.insert(position, file);
            Ok((self.cache.get_mut(&position).unwrap(), true))
        }
    }
}
