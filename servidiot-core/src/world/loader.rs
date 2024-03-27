use std::{collections::HashMap, path::PathBuf, thread::spawn};

use servidiot_anvil::{WorldManager, region::{RegionManager, RegionManagerError, file::ChunkError, nbt::ChunkRoot}};
use servidiot_primitives::{position::{RegionPosition, ChunkLocation, ChunkPosition}, chunk::{Chunk, section::ChunkSection}};

use super::TicketCount;

pub enum WorldLoaderCommand {
    LoadChunk(ChunkLocation),
    SaveChunk(ChunkLocation, ChunkRoot)
}

pub struct WorldLoader {
    world_manager: WorldManager,
    dimensions: HashMap<i32, (RegionManager, HashMap<RegionPosition, TicketCount>)>,

    loaded_channel: flume::Sender<(Chunk, ChunkLocation)>,
    command_recv: flume::Receiver<WorldLoaderCommand>
}

impl WorldLoader {
    pub fn create(folder: PathBuf) -> (flume::Sender<WorldLoaderCommand>, flume::Receiver<(Chunk, ChunkLocation)>) {
        let (command_send, command_recv) = flume::unbounded();
        let (chunk_send, chunk_recv) = flume::unbounded();

        let mut s = Self {
            world_manager: WorldManager::open(folder),
            dimensions: Default::default(),
            loaded_channel: chunk_send,
            command_recv
        };
        rayon::spawn(move || {
            s.run();
        });

        (command_send, chunk_recv)
    } 
    fn get_dimension(
        &mut self,
        id: i32,
    ) -> &mut (RegionManager, HashMap<RegionPosition, TicketCount>) {
        self.dimensions.entry(id).or_insert_with(|| {
            (
                self.world_manager.load_dimension(id).unwrap(),
                Default::default(),
            ) // TODO: propagate this error
        })
    }

    fn try_load_chunk(&mut self, position: ChunkLocation) -> anyhow::Result<()> {
        let mgr = &mut self.get_dimension(position.location.dimension).0;
        match mgr.load_chunk(position.position) {
            Ok(v) => {
                let chunk = chunk_root_to_chunk(&v.0);
                self.increment_ticket(position);
                let _ = self.loaded_channel.send((chunk, position));
                Ok(())
            }
            Err(RegionManagerError::ChunkError(ChunkError::ChunkNotPresent(_))) => {
                // need to generate it
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }


    fn increment_ticket(&mut self, position: ChunkLocation) {
        let dim = self.dimensions.get_mut(&position.location.dimension).expect("assumed loaded");
        dim.1.entry(position.position.region()).or_default().increment(); 
    }

    fn unload_chunk(&mut self, position: ChunkLocation, data: ChunkRoot) -> anyhow::Result<()> {
        let dim = self.dimensions.get_mut(&position.location.dimension).expect("assumed loaded");
        let region = position.position.region();
        dim.0.save_chunk(position.position, data)?;
        if let Some(ticket) = dim.1.get_mut(&region) {
            if ticket.decrement() {
                dim.1.remove(&region);
                dim.0.unload_region(region)?;
            }
        }
        if dim.1.is_empty() {
            dim.0.flush_cache()?;
            self.dimensions.remove(&position.location.dimension);
        }
        Ok(())
    }

    fn run(mut self) {

        while let Ok(command) = self.command_recv.recv() {
            match command {
                WorldLoaderCommand::LoadChunk(pos) => if let Err(e) = self.try_load_chunk(pos) {
                    tracing::error!("Chunk load failure: {:?}", e)
                },
                WorldLoaderCommand::SaveChunk(pos, data) => if let Err(e) = self.unload_chunk(pos, data) {
                    tracing::error!("Chunk save failure: {:?}", e)
                }
            }
        }

    }
}


fn chunk_root_to_chunk(c: &ChunkRoot) -> Chunk {
    let mut chunk = Chunk::new(ChunkPosition::new(c.level.x_position, c.level.z_position));

    for section in &c.level.sections {
        let mut new_sec = ChunkSection::empty(0);
        new_sec.block_light = section.block_light.clone();
        new_sec.block_meta = section.data.clone();
        new_sec.skylight = section.sky_light.clone();
        new_sec.block_types = section.blocks.as_u8_array().to_vec();
        new_sec.block_types_add = section.additional.clone();

        chunk.set_section(section.y_index as u8, new_sec)
    }
    chunk
}
