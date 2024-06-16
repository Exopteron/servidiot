use anyhow::bail;
use fxhash::FxHashMap;
use servidiot_anvil::{region::RegionManager, WorldManager};
use servidiot_primitives::position::{DimensionID, RegionPosition};

use super::{world::Command, Response};




pub(super) struct Dimension {
    manager: RegionManager,
    loaded_count: u32
}

impl Dimension {
    pub fn new(r: RegionManager) -> Self {
        Self {
            manager: r,
            loaded_count: 0
        }
    }
}


/// The task that manages a world.
pub(super) struct WorldTask {
    receiver: flume::Receiver<Command>,
    sender: flume::Sender<Response>,
    dimensions: FxHashMap<DimensionID, Dimension>
}

impl WorldTask {
    pub fn new(receiver: flume::Receiver<Command>, sender: flume::Sender<Response>, dimensions: FxHashMap<DimensionID, Dimension>) -> Self {
        Self {
            receiver,
            sender,
            dimensions
        }
    }

    pub fn run(mut self) -> anyhow::Result<()> {

        while let Ok(command) = self.receiver.recv() {
            match command {
                Command::Shutdown => break,
                Command::LoadChunk(location) => if let Some(dim) = self.dimensions.get_mut(&location.location.dimension) {
                    dim.loaded_count += 1;

                    let (chunk, _) = dim.manager.load_chunk(location.position)?;
                    self.sender.send(Response::LoadedChunk(location, chunk))?;
                } else {
                    bail!("Could not find dimension for chunk load request: {:?}", location)
                }
                Command::SaveChunk(location, data) => {
                    let should_remove;
                    if let Some(dim) = self.dimensions.get_mut(&location.location.dimension) {
                        dim.loaded_count -= 1;
                        should_remove = dim.loaded_count == 0;
                        
                        dim.manager.save_chunk(location.position, data)?;
                    } else {
                        bail!("Could not find dimension for chunk save request: {:?}", location)
                    }
                    if should_remove {
                        self.dimensions.remove(&location.location.dimension);
                    }
                }
            }
        }

        Ok(())
    }
}


