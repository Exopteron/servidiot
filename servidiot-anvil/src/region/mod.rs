use std::{
    fs::File,
    io,
    path::PathBuf,
    time::{SystemTime, SystemTimeError, UNIX_EPOCH, Duration},
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
    pub fn load_chunk(&mut self, position: ChunkPosition) -> RegionManagerResult<(ChunkRoot, SystemTime)> {
        self.load_region(position.region())?;
        let (data, time) = self.cache
            .get_mut(&position.region())
            .expect("should be present by this point")
            .read_chunk(position)
            .map_err(RegionManagerError::ChunkError)?;
        
        Ok((data, UNIX_EPOCH + Duration::from_secs(time as u64)))
    }

    /// Saves chunk data.
    pub fn save_chunk(
        &mut self,
        position: ChunkPosition,
        data: ChunkRoot,
    ) -> RegionManagerResult<()> {
        self.load_region(position.region())?;
        self.cache
            .get_mut(&position.region())
            .expect("should be present by this point")
            .write_chunk(
                self.compression_method,
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
            data.flush().map_err(RegionManagerError::IOError)?;
        }
        Ok(())
    }

    /// Removes a region from cache. Returns
    /// `true` if the region was present in cache.
    pub fn unload_region(&mut self, position: RegionPosition) -> RegionManagerResult<bool> {
        if let Some(mut v) = self.cache.remove(&position) {
            v.flush().map_err(RegionManagerError::IOError)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Caches a region. Returns `true` if
    /// the region was already cached.
    pub fn load_region(&mut self, position: RegionPosition) -> RegionManagerResult<bool> {
        if self.cache.contains_key(&position) {
            return Ok(true);
        }
        let mut path = self.directory.clone();
        path.push(format!("r.{}.{}.mca", position.x, position.z));

        if !path.try_exists().map_err(RegionManagerError::IOError)? {
            let file = RegionFile::create(
                File::options()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(path)
                    .map_err(RegionManagerError::IOError)?,
            )
            .map_err(RegionManagerError::IOError)?;
            self.cache.insert(position, file);
        } else {
            let file = RegionFile::open(
                File::options()
                    .read(true)
                    .write(true)
                    .open(path)
                    .map_err(RegionManagerError::IOError)?,
            )
            .map_err(RegionManagerError::IOError)?;
            self.cache.insert(position, file);
        }
        Ok(false)
    }
}

