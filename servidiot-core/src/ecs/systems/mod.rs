use servidiot_primitives::position::ChunkPosition;

use crate::{MinecraftServer, world::World};
mod network;

use super::{system::SystemExecutor, resources::Resources};

pub fn register_systems(s: &mut SystemExecutor<MinecraftServer>, resources: &mut Resources) {

    let mut world = World::new("".into());
    world.load_chunk(0, ChunkPosition::new(0, 0)).unwrap();

    s.group::<World>().add_system(|_state, world| {
        world.process()?;
        Ok(())
    });

    resources.insert(world);
    network::register_systems(s);
}

