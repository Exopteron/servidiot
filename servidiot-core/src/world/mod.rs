use std::path::PathBuf;

use ahash::AHashMap;
use servidiot_anvil::WorldManager;
use servidiot_primitives::{position::ChunkPosition, chunk::{store::ChunkStore, handle::ChunkHandle, Chunk}};

use self::{region::{RegionThread, IdentifiedResponse, ChunkLoadResult, RegionThreadCommand}};

mod region;
mod chunk;
/// A Minecraft world.
pub struct World {
    anvil: servidiot_anvil::WorldManager,
    dimensions: AHashMap<i32, (RegionThread, ChunkStore)>,


    recv: flume::Receiver<IdentifiedResponse>,
    send: flume::Sender<IdentifiedResponse>
}

impl World {
    pub fn new(directory: PathBuf) -> Self {
        let (send, recv) = flume::unbounded();
        Self {
            anvil: WorldManager::open(directory),
            dimensions: AHashMap::default(),
            send,
            recv
        }
    }

    /// Process chunk loads.
    pub fn process(&mut self) -> anyhow::Result<()> {
        for command in self.recv.try_iter().collect::<Vec<_>>() {
            match command.command {
                region::RegionThreadResponse::ChunkLoaded(pos, res) => {
                    match res {
                        ChunkLoadResult::Success(v) => {
                            log::info!("Loaded chunk {pos} successfully");
                            self.get_dimension(command.dim_id)?.1.add_chunk(pos, ChunkHandle::new(*v, true));
                        }
                        ChunkLoadResult::Fail(error) => log::error!("chunk error: {error:?}")
                    }
                },
            }
        }
        Ok(())
    }

    /// Schedules a chunk load. Returns `true` if the chunk is already loaded.
    pub fn load_chunk(&mut self, dimension: i32, position: ChunkPosition) -> anyhow::Result<bool> {
        let (thread, dim) = self.get_dimension(dimension)?;
        if dim.get_chunk(position).is_ok() {
            return Ok(true);
        }
        thread.send_command(RegionThreadCommand::LoadChunk(position))?;
        Ok(false)
    }


    /// Saves a chunk.
    pub fn save_chunk(&mut self, dimension: i32, position: ChunkPosition, chunk: Chunk) -> anyhow::Result<()> {
        let (thread, _) = self.get_dimension(dimension)?;
        thread.send_command(RegionThreadCommand::SaveChunk(position, Box::new(chunk)))
    }


    pub fn dimension_handle(&mut self, dimension: i32) -> anyhow::Result<ChunkStore> {
        let (_, dim) = self.get_dimension(dimension)?;
        Ok(dim.new_handle())
    }


    fn get_dimension(&mut self, dimension: i32) -> anyhow::Result<&(RegionThread, ChunkStore)> {
        self.load_dimension(dimension)?;
        Ok(self.dimensions.get(&dimension).unwrap())
    }

    /// Loads a dimension. Returns `true` if it
    /// was already loaded.
    fn load_dimension(&mut self, id: i32) -> anyhow::Result<bool> {
        if self.dimensions.contains_key(&id) {
            return Ok(true);
        }
        let man = self.anvil.load_dimension(id)?;
        let thread = RegionThread::create(man, id, self.send.clone());
        self.dimensions.insert(id, (thread, ChunkStore::new()));
        Ok(false)
    } 
}