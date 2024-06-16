use fxhash::FxHashMap;
use rayon::ThreadPool;
use servidiot_anvil::{region::nbt::ChunkRoot, WorldManager};
use servidiot_primitives::{chunk::Chunk, position::{ChunkLocation, DimensionID}};

use crate::world::thread::{Dimension, WorldTask};

pub(super) enum Command {
    Shutdown,
    LoadChunk(ChunkLocation),
    SaveChunk(ChunkLocation, ChunkRoot)
}

pub(super) enum Response {
    LoadedChunk(ChunkLocation, ChunkRoot)
}

/// One world in a collection of worlds.
pub(super) struct World {
    manager: WorldManager,
    commands: flume::Sender<Command>,
}

impl World {

    pub fn request_chunk_load(&self, location: ChunkLocation) -> anyhow::Result<()> {
        self.commands.send(Command::LoadChunk(location))?;
        Ok(())
    }

    pub fn request_chunk_save(&self, location: ChunkLocation, data: ChunkRoot) -> anyhow::Result<()> {
        self.commands.send(Command::SaveChunk(location, data))?;
        Ok(())
    }


    pub fn new(thread: &ThreadPool, world: WorldManager, dimensions_to_load: &[DimensionID], channel: flume::Sender<Response>) -> anyhow::Result<Self> {
        

        let (send, recv) = flume::bounded(8);


        let mut dimensions = FxHashMap::default();
        for dim in dimensions_to_load {
            dimensions.insert(*dim, Dimension::new(world.load_dimension(*dim)?));
        }

        let task = WorldTask::new(recv, channel, dimensions);
        
        thread.spawn(move || {
            if let Err(e) = task.run() {
                tracing::error!("world manager err: {:?}", e)
            }
        });

        Ok(Self {
            manager: world,
            commands: send
        })
    }
}