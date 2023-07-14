use std::sync::Arc;



use ahash::AHashMap;
use anyhow::bail;
use parking_lot::RwLock;

use crate::position::ChunkPosition;

use super::handle::ChunkHandle;

struct ChunkStoreInner {
    chunks: AHashMap<ChunkPosition, ChunkHandle>
}

/// A chunk store.
pub struct ChunkStore(Arc<RwLock<ChunkStoreInner>>);

impl Default for ChunkStore {
    fn default() -> Self {
        Self::new()
    }
}


impl ChunkStore {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(ChunkStoreInner { chunks: Default::default() })))
    }

    pub fn chunks(&self) -> Vec<ChunkPosition> {
        self.0.read().chunks.keys().copied().collect::<Vec<_>>()
    }

    pub fn remove_chunk(&self, position: ChunkPosition) -> anyhow::Result<ChunkHandle> {
        match self.0.write().chunks.remove(&position) {
            Some(v) => Ok(v),
            None => bail!("{position} not present")
        }
    }

    pub fn get_chunk(&self, position: ChunkPosition) -> anyhow::Result<ChunkHandle> {
        match self.0.read().chunks.get(&position).cloned() {
            Some(v) => Ok(v),
            None => bail!("{position} not present")
        }
    }

    pub fn add_chunk(&self, position: ChunkPosition, handle: ChunkHandle) {
        self.0.write().chunks.insert(position, handle);
    }

    pub fn new_handle(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}