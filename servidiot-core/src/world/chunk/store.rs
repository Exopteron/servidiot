use std::sync::Arc;

use ahash::AHashMap;
use parking_lot::RwLock;
use servidiot_primitives::position::{ChunkPosition};

use super::handle::ChunkHandle;

struct ChunkStoreInner {
    chunks: AHashMap<ChunkPosition, ChunkHandle>
}

/// A chunk store.
pub struct ChunkStore(Arc<RwLock<ChunkStoreInner>>);

impl ChunkStore {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(ChunkStoreInner { chunks: Default::default() })))
    }

    pub fn get_chunk(&self, position: ChunkPosition) -> Option<ChunkHandle> {
        self.0.read().chunks.get(&position).cloned()
    }

    pub fn add_chunk(&self, position: ChunkPosition, handle: ChunkHandle) {
        self.0.write().chunks.insert(position, handle);
    }

    pub fn new_handle(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}