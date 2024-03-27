use servidiot_ecs::SystemExecutor;
use servidiot_network::server::Server;

use crate::{game::GameState, world::GameWorld};

pub fn register_systems(s: &mut SystemExecutor<GameState>) {
    s.add_system(process_chunk_loads);
}

pub fn process_chunk_loads(state: &GameState) -> anyhow::Result<()> {
    let mut world = state.resources().get_mut::<GameWorld>();
    let server = state.resources().get::<Server>();

    world.process_loads(&server)
}