use servidiot_ecs::SystemExecutor;
use servidiot_network::server::{Server, id::NetworkID};
use servidiot_primitives::position::{ChunkLocation, EntityLocation};

use crate::{game::GameState, events::entity::PlayerViewChangeEvent, world::GameWorld};

pub fn register_systems(s: &mut SystemExecutor<GameState>) {
    s.add_system(handle_view_change);
}

pub fn handle_view_change(state: &GameState) -> anyhow::Result<()> {

    let ecs = state.ecs().borrow();
    let server = state.resources().get::<Server>();
    let mut world = state.resources().get_mut::<GameWorld>();
    for e in state.events().borrow().deferred_events::<PlayerViewChangeEvent>() {
        let player = ecs.entity(e.entity)?;
        let client = server.get_client(*player.get::<&NetworkID>().unwrap())?;

        let loc = player.get::<&EntityLocation>().unwrap().location;

        {
            let mut waiting_on = client.client_waiting_chunks.lock();
            for v in waiting_on.clone() {
                if !e.new_view.contains(v.position) {
                    waiting_on.remove(&v);
                    world.cancel_loading_request(v, client.id);
                }
            }
        }
        
        for v in e.new_view.chunks() {
            if !e.old_view.contains(v) {

                world.add_player_to_chunk(client, e.entity, ChunkLocation {
                    position: v,
                    location: loc
                })?;
            }
        }

        for v in e.old_view.chunks() {
            if !e.new_view.contains(v) {
                world.remove_player_from_chunk(client, e.entity, ChunkLocation {
                    position: v,
                    location: loc
                })?;
            }
        }

    }

    Ok(())
}
