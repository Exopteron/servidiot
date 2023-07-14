use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use anyhow::bail;
use parking_lot::{RwLockReadGuard, RwLock, RwLockWriteGuard};
use thiserror::Error;


use super::Chunk;



struct ChunkHandleInner {
    chunk: RwLock<Chunk>,
    is_loaded: AtomicBool
}

pub struct ChunkHandle(Arc<ChunkHandleInner>);

#[derive(Error, Debug)]
pub enum ChunkHandleError {
    #[error("chunk is not loaded")]
    Unloaded
}

pub type ChunkHandleResult<T> = std::result::Result<T, ChunkHandleError>;

impl ChunkHandle {
    pub fn new(chunk: Chunk, is_loaded: bool) -> Self {
        Self(Arc::new(ChunkHandleInner { chunk: RwLock::new(chunk), is_loaded: AtomicBool::new(is_loaded) }))
    }

    /// Attempt to remove the chunk from its handle.
    pub fn take(self) -> std::result::Result<Chunk, Self> {
        match Arc::try_unwrap(self.0) {
            Ok(v) => Ok(v.chunk.into_inner()),
            Err(v) => Err(Self(v)),
        }
    }

    /// Read access to the chunk. 
    pub fn chunk(&self) -> RwLockReadGuard<Chunk> {
        self.0.chunk.read()
    }


    /// Write access to the chunk. Returns 
    /// `None` if the chunk is unloaded.
    pub fn chunk_mut(&mut self) -> ChunkHandleResult<RwLockWriteGuard<Chunk>> {
        if !self.is_loaded() {
            Err(ChunkHandleError::Unloaded)
        } else {
            Ok(self.0.chunk.write())
        }
    }

    /// Creates a new handle to this chunk.
    pub fn new_handle(&self) -> ChunkHandle {
        Self(Arc::clone(&self.0))
    }
    /// Is this chunk loaded?
    pub fn is_loaded(&self) -> bool {
        self.0.is_loaded.load(Ordering::SeqCst)
    }

    pub fn set_unloaded(&self) -> anyhow::Result<()> {
        if self.0.is_loaded.swap(false, Ordering::SeqCst) {
            // FIXME: Decide what to do when unloading an unloaded chunk
        }
        if self.0.chunk.try_read().is_none() {
            // Locking fails when someone else already owns a write lock
            bail!("Cannot unload chunk because it is locked as writable!")
        }
        Ok(())
    }

    pub fn set_loaded(&self) -> bool {
        self.0.is_loaded.swap(true, Ordering::SeqCst)
    }
}

impl Clone for ChunkHandle {
    fn clone(&self) -> Self {
        self.new_handle()
    }
}