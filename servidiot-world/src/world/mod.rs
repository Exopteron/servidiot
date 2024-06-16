mod thread;
mod world;
use std::path::PathBuf;

use anyhow::bail;
use flume::{Receiver, Sender};
use fxhash::FxHashMap;
use rayon::ThreadPool;
use servidiot_anvil::{region::nbt::ChunkRoot, WorldManager};
use servidiot_primitives::position::{ChunkLocation, DimensionID};
use world::{Response, World};







/// A collection of Minecraft worlds, with multiworld support.
pub struct Worlds {
    thread_pool: ThreadPool,
    multiworld_entries: FxHashMap<u32, World>,

    thread_commands: (Sender<Response>, Receiver<Response>),
}

impl Worlds {

    pub fn process_loads(&self) -> Vec<(ChunkLocation, ChunkRoot)> {
        let mut loaded = vec![];
        for v in self.thread_commands.1.try_iter() {
            match v {
                Response::LoadedChunk(loc, data) => loaded.push((loc, data))
            }
        }
        loaded
    }

    pub fn request_chunk_load(&self, location: ChunkLocation) -> anyhow::Result<()> {
        if let Some(entry) = self.multiworld_entries.get(&location.location.world) {
            entry.request_chunk_load(location)
        } else {
            bail!("multiworld world not present: {:?}", location)
        }
    }

    pub fn request_chunk_save(&self, location: ChunkLocation, data: ChunkRoot) -> anyhow::Result<()> {
        if let Some(entry) = self.multiworld_entries.get(&location.location.world) {
            entry.request_chunk_save(location, data)
        } else {
            bail!("multiworld world not present: {:?}", location)
        }
    }

    pub fn new(pool: ThreadPool) -> Self {
        Self {
            thread_pool: pool,
            multiworld_entries: Default::default(),
            thread_commands: flume::unbounded(),
        }
    }

    /// Add a multiworld world to this manager.
    pub fn add(
        &mut self,
        id: u32,
        dimensions: &[DimensionID],
        directory: PathBuf,
    ) -> anyhow::Result<()> {
        let world = World::new(
            &self.thread_pool,
            WorldManager::open(directory),
            dimensions,
            self.thread_commands.0.clone(),
        )?;

        self.multiworld_entries.insert(id, world);
        Ok(())
    }
}
