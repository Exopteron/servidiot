use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use servidiot_primitives::chunk::Chunk;


struct ChunkHandleInner {
    chunk: RwLock<Chunk>,
    is_loaded: AtomicBool
}

pub struct ChunkHandle(Arc<ChunkHandleInner>);

impl ChunkHandle {
    pub fn new(chunk: Chunk, is_loaded: bool) -> Self {
        Self(Arc::new(ChunkHandleInner { chunk: RwLock::new(chunk), is_loaded: AtomicBool::new(is_loaded) }))
    }

    /// Read access to the chunk. Returns `None`
    /// if the chunk is unloaded.
    pub fn chunk(&self) -> Option<RwLockReadGuard<Chunk>> {
        if !self.is_loaded() {
            None
        } else {
            Some(self.0.chunk.read())
        }
    }


    /// Write access to the chunk. Returns 
    /// `None` if the chunk is unloaded.
    pub fn chunk_mut(&mut self) -> Option<RwLockWriteGuard<Chunk>> {
        if !self.is_loaded() {
            None
        } else {
            Some(self.0.chunk.write())
        }
    }

    /// Creates a new handle to this chunk.
    pub fn new_handle(&self) -> ChunkHandle {
        Self(Arc::clone(&self.0))
    }
    /// Is this chunk loaded?
    pub fn is_loaded(&self) -> bool {
        self.0.is_loaded.load(Ordering::Acquire)
    }

    pub fn set_loaded(&self, value: bool) {
        self.0.is_loaded.store(value, Ordering::Release);
    }
}

impl Clone for ChunkHandle {
    fn clone(&self) -> Self {
        self.new_handle()
    }
}