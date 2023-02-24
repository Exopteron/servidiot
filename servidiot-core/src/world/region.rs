use std::thread::JoinHandle;

use servidiot_anvil::region::{RegionManager, nbt::ChunkRoot, RegionManagerError, file::ChunkError};
use servidiot_primitives::{position::ChunkPosition, chunk::{Chunk, section::ChunkSection}};
use thiserror::Error;


pub struct RegionThread {
    dimension: i32,
    commands: flume::Sender<RegionThreadCommand>,
    thread: JoinHandle<()>
}

impl RegionThread {
    pub fn send_command(&self, command: RegionThreadCommand) -> anyhow::Result<()> {
        self.commands.send(command)?;
        Ok(())
    }

    pub fn create(mut region_manager: RegionManager, dimension: i32, response_sender: flume::Sender<IdentifiedResponse>) -> Self {
        let (command_sender, command_recv) = flume::unbounded();
        let thread = std::thread::spawn(move || {
            for value in command_recv.into_iter() {
                match value {
                    RegionThreadCommand::Shutdown => {
                        if let Err(e) = region_manager.flush_cache() {
                            log::error!("Region error: {:?}", e);
                        }
                        break;
                    }
                    RegionThreadCommand::LoadChunk(c) => {
                        if let Err(error) = match region_manager.load_chunk(c) {
                            Ok(v) => response_sender.send(RegionThreadResponse::ChunkLoaded(c, chunk_root_to_chunk(c, v.0)).id(dimension)),
                            Err(RegionManagerError::ChunkError(ChunkError::ChunkNotPresent(_))) => {
                                todo!() // generate chunk
                            }
                            Err(e) => {
                                response_sender.send(RegionThreadResponse::ChunkLoaded(c, ChunkLoadResult::Fail(ChunkLoadError::RegionError(e))).id(dimension))
                            }
                        } {
                            log::error!("Chunk thread error: {error:?}");
                            break;
                        }
                    }
                }
            }
        });
        Self {
            commands: command_sender,
            thread,
            dimension
        }
    }
}

pub enum RegionThreadCommand {
    Shutdown,
    LoadChunk(ChunkPosition)
}


pub struct IdentifiedResponse {
    pub dim_id: i32,
    pub command: RegionThreadResponse
}

pub enum RegionThreadResponse {
    ChunkLoaded(ChunkPosition, ChunkLoadResult)
}

#[derive(Debug, Error)]
pub enum ChunkLoadError {
    #[error("region error: {0}")]
    RegionError(RegionManagerError),
    #[error("chunk {0} reported as being at ({1}, {2})")]
    PositionMismatch(ChunkPosition, i32, i32)
}

pub enum ChunkLoadResult {
    Success(Box<Chunk>),
    Fail(ChunkLoadError)
}

impl RegionThreadResponse {
    pub fn id(self, id: i32) -> IdentifiedResponse {
        IdentifiedResponse { dim_id: id, command: self }
    }
}

/// Transform chunk NBT to an internal chunk.
fn chunk_root_to_chunk(pos: ChunkPosition, root: ChunkRoot) -> ChunkLoadResult {
    if root.level.x_position != pos.x || root.level.z_position != pos.z {
        return ChunkLoadResult::Fail(ChunkLoadError::PositionMismatch(pos, root.level.x_position, root.level.z_position))
    }
    let mut chunk = Chunk::new(pos);
    if let Some(root_biomes) = root.level.biomes {
        for x in 0..Chunk::LENGTH {
            for z in 0..Chunk::WIDTH {
                let biome_index = (z * 16) + x;
                chunk.biomes_mut()[x][z] = root_biomes[biome_index] as u8;
            }
        }
    }

    for section in root.level.sections {
        let id = section.y_index as u8;
        let section = ChunkSection {
            section_id: id,
            skylight: section.sky_light,
            block_light: section.block_light,
            block_types: unsafe { // safe because Vec<i8> and Vec<u8> should have the same rep
                std::mem::transmute(section.blocks.0)
            },
            block_types_add: section.additional,
            block_meta: section.data
        };
        chunk.set_section(id, section);
    }
    ChunkLoadResult::Success(Box::new(chunk))
}