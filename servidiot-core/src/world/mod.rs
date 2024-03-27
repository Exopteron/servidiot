use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use servidiot_ecs::Entity;
use servidiot_network::server::{id::NetworkID, Client, Server};
use servidiot_primitives::{
    chunk::{Chunk, ChunkBitmap},
    position::ChunkLocation,
};


use self::loader::{WorldLoader, WorldLoaderCommand};

mod loader;
pub mod view;

#[derive(Default)]
pub struct TicketCount(pub usize);

impl TicketCount {
    pub fn increment(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    pub fn decrement(&mut self) -> bool {
        self.0 = self.0.saturating_sub(1);
        self.0 == 0
    }
}

pub struct GameWorld {
    loading_requests: HashMap<ChunkLocation, HashMap<NetworkID, Entity>>,

    command_sender: flume::Sender<WorldLoaderCommand>,
    chunk_recv: flume::Receiver<(Chunk, ChunkLocation)>,

    chunks: HashMap<ChunkLocation, (Chunk, TicketCount, HashSet<Entity>)>, 
}

impl GameWorld {
    pub fn new(folder: PathBuf) -> Self {
        let (loaded, recv) = WorldLoader::create(folder);
        Self {
            loading_requests: Default::default(),
            chunks: Default::default(),
            command_sender: loaded,
            chunk_recv: recv,
        }
    }

    /// Returns `None` if the chunk is not loaded.
    pub fn get_chunk(&self, loc: ChunkLocation) -> Option<&(Chunk, TicketCount, HashSet<Entity>)> {
        self.chunks.get(&loc)
    }

    /// Returns `None` if the chunk is not loaded.
    pub fn get_chunk_mut(
        &mut self,
        loc: ChunkLocation,
    ) -> Option<&mut (Chunk, TicketCount, HashSet<Entity>)> {
        self.chunks.get_mut(&loc)
    }

    fn add_chunk(&mut self, position: ChunkLocation, chunk: Chunk) {
        self.chunks.insert(
            position,
            (chunk, TicketCount(0), Default::default()),
        );
    }

    fn add_ticket_for_loaded(&mut self, chunk: ChunkLocation) {
        let chunk_data = self
            .get_chunk_mut(chunk)
            .expect("Should be loaded when this is called");
        chunk_data.1.increment();
    }

    /// Returns `false` if loading of this chunk has been deferred.
    pub fn add_ticket(&mut self, chunk: ChunkLocation) -> anyhow::Result<bool> {
        if self.get_chunk(chunk).is_none() {
            let _ = self
                .command_sender
                .send(WorldLoaderCommand::LoadChunk(chunk));
            Ok(false)
        } else {
            self.add_ticket_for_loaded(chunk);
            Ok(true)
        }
    }

    pub fn is_loaded(&self, chunk: ChunkLocation) -> bool {
        self.get_chunk(chunk).is_some()
    }


    fn save_chunk(&mut self, chunk: ChunkLocation) -> anyhow::Result<()> {
        if let Some(c) = self.chunks.remove(&chunk) {
            tracing::error!("Chunk saving not done");
            //self.command_sender.send(WorldLoaderCommand::SaveChunk(chunk, todo!()))?;
        }
        Ok(())
    }

    pub fn remove_ticket(&mut self, chunk: ChunkLocation, entity: Option<Entity>) -> anyhow::Result<()> {

        let chunk_data = self
            .get_chunk_mut(chunk)
            .expect("Should be loaded when this is called");

        if let Some(entity) = entity {
            chunk_data.2.remove(&entity);
        }

        if chunk_data.1.decrement() {
            self.save_chunk(chunk)?;
        }
        Ok(())
    }

    pub fn remove_player_from_chunk(
        &mut self,
        player: &Client,
        player_entity: Entity,
        chunk: ChunkLocation,
    ) -> anyhow::Result<()> {
        if self.is_loaded(chunk) {
            self.remove_ticket(chunk, Some(player_entity))?;
        }
        if !player.is_disconnected() {
            {
                player.client_known_chunks.lock().remove(&chunk.position);
            }
            player.unload_chunk(chunk.position)?;
        }
        Ok(())
    }

    fn load_for_client(
        &mut self,
        player: &Client,
        player_entity: Entity,
        chunk: ChunkLocation,
    ) -> anyhow::Result<()> {
        let chunk_data = self
            .get_chunk_mut(chunk)
            .expect("Should be loaded when this is called");

        chunk_data.2.insert(player_entity);
        {
            player.client_waiting_chunks.lock().remove(&chunk);
            player.client_known_chunks.lock().insert(chunk.position);
        }
        player.send_chunk(&chunk_data.0, ChunkBitmap::full())?;

        Ok(())
    }

    pub fn add_player_to_chunk(
        &mut self,
        player: &Client,
        player_entity: Entity,
        chunk: ChunkLocation,
    ) -> anyhow::Result<()> {
        if self.add_ticket(chunk)? {
            self.load_for_client(player, player_entity, chunk)?;
        } else {
            {
                player.client_waiting_chunks.lock().insert(chunk);
            }
            self.loading_requests
                .entry(chunk)
                .or_default()
                .insert(player.id, player_entity);
        }
        Ok(())
    }

    pub fn cancel_loading_request(&mut self, chunk: ChunkLocation, id: NetworkID) {
        if let Some(v) = self
            .loading_requests
            .get_mut(&chunk)
        {
            v.remove(&id);
        }
    }

    pub fn process_loads(&mut self, server: &Server) -> anyhow::Result<()> {
        while let Ok((chunk, location)) = self.chunk_recv.try_recv() {
            self.add_chunk(location, chunk);
            if let Some(requests) = self.loading_requests.remove(&location) {
                for req in requests {
                    let cl = server.get_client(req.0)?;
                    self.add_player_to_chunk(cl, req.1, location)?;
                }
            }
        }
        Ok(())
    }
}
