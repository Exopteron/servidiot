use std::{fs::File, io, path::PathBuf, time::SystemTime};

use crate::nbt::level::LevelRoot;
use ::nbt::{from_gzip_reader, to_gzip_writer};
use ahash::AHashMap;
use region::{file::CompressionType, nbt::ChunkRoot, RegionManager, RegionManagerError};
use servidiot_primitives::position::ChunkPosition;
use thiserror::Error;

pub mod nbt;
pub mod region;

/// Represents a world folder.
pub struct WorldManager {
    /// The world directory.
    directory: PathBuf,
    /// Dimension cache.
    dimensions: AHashMap<i32, RegionManager>,
}

#[derive(Error, Debug)]
pub enum WorldManagerError {
    #[error("IO error: {0}")]
    IOError(io::Error),
    #[error("NBT error: {0}")]
    NBTError(::nbt::Error),
    #[error("Region error: {0}")]
    RegionError(RegionManagerError),
}
pub type WorldManagerResult<T> = std::result::Result<T, WorldManagerError>;

impl WorldManager {
    pub fn open(directory: PathBuf) -> Self {
        Self {
            dimensions: Default::default(),
            directory,
        }
    }

    /// Flush all caches.
    pub fn flush_cache(&mut self) -> WorldManagerResult<()> {
        for (_, mut v) in std::mem::take(&mut self.dimensions) {
            v.flush_cache().map_err(WorldManagerError::RegionError)?;
        }
        Ok(())
    }

    /// Loads a dimension. Returns `true` if it
    /// was already loaded.
    fn load_dimension(&mut self, dimension: i32) -> WorldManagerResult<bool> {
        if self.dimensions.contains_key(&dimension) {
            return Ok(true);
        }
        let mut dir = self.directory.clone();
        if dimension != 0 {
            dir.push(format!("DIM{dimension}"));
        }
        dir.push("region");
        std::fs::create_dir_all(&dir).map_err(WorldManagerError::IOError)?;
        self.dimensions
            .insert(dimension, RegionManager::new(dir, CompressionType::ZLib));
        Ok(false)
    }

    /// Saves a chunk.
    pub fn save_chunk(
        &mut self,
        dimension: i32,
        position: ChunkPosition,
        data: ChunkRoot,
    ) -> WorldManagerResult<()> {
        self.load_dimension(dimension)?;
        let Some(dim) = self.dimensions.get_mut(&dimension) else {
            unreachable!()
        };
        dim.save_chunk(position, data)
            .map_err(WorldManagerError::RegionError)
    }

    /// Loads a chunk.
    pub fn load_chunk(
        &mut self,
        dimension: i32,
        position: ChunkPosition,
    ) -> WorldManagerResult<(ChunkRoot, SystemTime)> {
        self.load_dimension(dimension)?;
        let Some(dim) = self.dimensions.get_mut(&dimension) else {
            unreachable!()
        };
        dim.load_chunk(position)
            .map_err(WorldManagerError::RegionError)
    }

    /// Load the level.dat from disk. Returns `None`
    /// if it is not present.
    pub fn load_level_dat(&self) -> WorldManagerResult<Option<LevelRoot>> {
        let mut dir = self.directory.clone();
        dir.push("level.dat");
        if !dir.exists() {
            return Ok(None);
        }
        let file = File::open(dir).map_err(WorldManagerError::IOError)?;
        let v = from_gzip_reader(file).map_err(WorldManagerError::NBTError)?;
        Ok(Some(v))
    }

    /// Save the level.dat to disk.
    pub fn save_level_dat(&mut self, value: &LevelRoot) -> WorldManagerResult<()> {
        let mut dir = self.directory.clone();
        dir.push("level.dat");
        let mut file = File::options()
            .write(true)
            .create(true)
            .open(dir)
            .map_err(WorldManagerError::IOError)?;
        to_gzip_writer(&mut file, &value, None).map_err(WorldManagerError::NBTError)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use servidiot_primitives::position::BlockPosition;

    use crate::{
        WorldManager,
    };

    #[test]
    pub fn epic_test() {
        let mut file = WorldManager::open(
            PathBuf::from_str(
                "",
            )
            .unwrap(),
        );
        println!("Level.dat: {:#?}", file.load_level_dat().unwrap());

        let block = BlockPosition::new(-1759, 5, 459);

        let (mut data, _) = file.load_chunk(0, block.chunk()).unwrap();
        println!("X{:?} Z{:?}", data.level.x_position, data.level.z_position);
        let section = &mut data.level.sections[(block.y / 16) as usize];

        // (y * 16 + z) * 16 + x
        // let x = (block.x & 15) as usize;
        // let y = (block.y & 15) as usize;
        // let z = (block.z & 15) as usize;
        //let position = (y * 16 + z) * 16 + x;

        section.blocks.fill(137u8 as i8);
        file.save_chunk(0, block.chunk(), data).unwrap();
        file.flush_cache().unwrap();
    }
}
