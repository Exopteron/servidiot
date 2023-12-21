use std::{fs::File, io, path::PathBuf};

use crate::nbt::level::LevelRoot;
use crate::nbt::player::PlayerData;
use ::nbt::{from_gzip_reader, to_gzip_writer};
use region::{file::CompressionType, RegionManager, RegionManagerError};
use thiserror::Error;
use uuid::Uuid;

pub mod nbt;
pub mod region;

/// Represents a world folder.
pub struct WorldManager {
    /// The world directory.
    directory: PathBuf,
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
            directory,
        }
    }


    /// Attempt to load playerdata for some UUID.
    pub fn load_player_data(&self, uuid: &Uuid) -> WorldManagerResult<Option<PlayerData>> {
        let mut dir = self.directory.clone();
        dir.push(format!("{}.dat", uuid.as_hyphenated()));
        if !dir.try_exists().map_err(WorldManagerError::IOError)? {
            return Ok(None);
        }
        let file = File::open(dir).map_err(WorldManagerError::IOError)?;
        let v = from_gzip_reader(file).map_err(WorldManagerError::NBTError)?;
        Ok(Some(v))
    } 


    /// Save playerdata to disk.
    pub fn save_player_data(&mut self, uuid: &Uuid, value: &PlayerData) -> WorldManagerResult<()> {
        let mut dir = self.directory.clone();
        dir.push(format!("{}.dat", uuid.as_hyphenated()));
        let mut file = File::options()
            .write(true)
            .create(true)
            .open(dir)
            .map_err(WorldManagerError::IOError)?;
        to_gzip_writer(&mut file, &value, None).map_err(WorldManagerError::NBTError)?;
        Ok(())
    }



    /// Loads a dimension.
    pub fn load_dimension(&mut self, dimension: i32) -> WorldManagerResult<RegionManager> {
        let mut dir = self.directory.clone();
        if dimension != 0 {
            dir.push(format!("DIM{dimension}"));
        }
        dir.push("region");
        std::fs::create_dir_all(&dir).map_err(WorldManagerError::IOError)?;
        Ok(RegionManager::new(dir, CompressionType::ZLib))
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
    use uuid::Uuid;

    use crate::WorldManager;

    #[test]
    pub fn epic_test() {
        // let mut file = WorldManager::open(
        //     PathBuf::from_str(
        //         "",
        //     )
        //     .unwrap(),
        // );

        // let mut world = file.load_dimension(0).unwrap();
        // println!("Level.dat: {:#?}", file.load_level_dat().unwrap());

        // let block = BlockPosition::new(-1759, 5, 459);


        // let (mut data, _) = world.load_chunk(block.chunk()).unwrap();
        // println!("X{:?} Z{:?}", data.level.x_position, data.level.z_position);
        // let section = &mut data.level.sections[(block.y / 16) as usize];

        // // (y * 16 + z) * 16 + x
        // // let x = (block.x & 15) as usize;
        // // let y = (block.y & 15) as usize;
        // // let z = (block.z & 15) as usize;
        // //let position = (y * 16 + z) * 16 + x;

        // section.blocks.fill(137u8 as i8);
        // world.save_chunk(block.chunk(), data).unwrap();
        // world.flush_cache().unwrap();
    }
}
