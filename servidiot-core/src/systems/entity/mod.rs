use servidiot_ecs::SystemExecutor;
use servidiot_network::server::{id::NetworkID, Client, Server};
use servidiot_primitives::{chunk, position::{EntityLocation, ChunkLocation}};

use crate::{game::GameState, world::{GameWorld, view::View}, events::entity::EntityMoveEvent, entity::{player::PlayerMarker, EntityDispatch}};

pub mod player;
pub fn register_systems(s: &mut SystemExecutor<GameState>) {
    player::register_systems(s);
    s.add_system(handle_entity_move);
}

pub fn handle_entity_move(state: &GameState) -> anyhow::Result<()> {

    let ecs = state.ecs().borrow();
    for e in state.events().borrow().deferred_events::<EntityMoveEvent>() {

        let this_entity = ecs.entity(e.entity)?;
    
        let loc = this_entity.get::<&EntityLocation>().unwrap().location;
        let this_id = *this_entity.get::<&NetworkID>().unwrap();


        

        let old_view = View::new(e.old_pos.chunk(), 8);
        let new_view = View::new(e.new_pos.chunk(), 8);

        let old_chunks = old_view.chunks();
        let new_chunks = new_view.chunks();

        let server = state.resources().get::<Server>();
        let world = state.resources().get::<GameWorld>();
        if e.old_pos.chunk() != e.new_pos.chunk() {
            // crossed a chunk boundary

            {


                let we_are_player = this_entity.has::<PlayerMarker>();
                if we_are_player {
                    state.unload_entities_for(&ecs, &server, &world, this_entity, loc, old_chunks.difference(&new_chunks).copied())?;
                }

                state.load_entities_around(&ecs, &server, &world, this_entity, loc, new_chunks.difference(&old_chunks).copied())?;
            }

        }

        state.for_all_entities_nearby(&ecs, &world, loc, new_chunks.iter().copied(), |other| {
            if other.entity() == this_entity.entity() {
                return Ok(());
            }
            if other.has::<PlayerMarker>() {
                let other_id = *other.get::<&NetworkID>().unwrap();
                let other = server.get_client(other_id)?;
                other.send_position(this_id, e.new_pos)?;
            }
            Ok(())
        })?;
    
    }

    Ok(())
}