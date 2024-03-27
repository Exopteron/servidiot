use servidiot_ecs::{EntityBuilder, SystemExecutor};
use servidiot_network::{server::Server, io::packet::client::play::ClientSettings};
use servidiot_primitives::{
    player::{Gamemode, GamemodeType},
    position::{ChunkLocation, ChunkPosition, Location, Position, EntityLocation},
};

use crate::{
    entity::{player::{PlayerEntity, PlayerMarker}, EntityDispatch},
    game::{GameState, ClientMap},
    world::{GameWorld, view::View},
};

pub fn register_systems(s: &mut SystemExecutor<GameState>) {
    s.add_system(handle_new_clients)
        .add_system(handle_disconnected_clients);
}

pub fn handle_new_clients(state: &GameState) -> anyhow::Result<()> {
    let mut sync_entities = vec![];
    {
        let mut server = state.resources().get_mut::<Server>();
        let mut world = state.resources().get_mut::<GameWorld>();
        let mut map = state.resources().get_mut::<ClientMap>();
    
    
        for client_id in server.accept_clients() {
            let client = server.get_client(client_id)?;
            tracing::info!("New client connected: {:?}", client.profile.name);
    
    
            let position = Position::new(0.0, 128.0, 0.0, 0.0, 0.0, false);
    
            let settings = ClientSettings {
                locale: "en-US".to_string(),
                view_distance: 4,
                chat_flags: 0,
                chat_colours: true,
                difficuty: 0,
                show_cape: true,
            };
    
            let view = View::new(position.chunk(), settings.view_distance as u32);
            
            let mut builder = EntityBuilder::new();
            builder.add(client_id);
            builder.add(PlayerMarker);
            builder.add(client.profile.clone());
            builder.add(EntityDispatch::new(PlayerEntity));
            builder.add(EntityLocation {
                position,
                location: Location::new(0, 0)
            });
            builder.add(settings);
    
    
    
    
            let id = state.ecs().borrow_mut().spawn(builder.build());
    
            map.add_mapping(client.id, id);
    
            client.join_game(
                Gamemode::new(GamemodeType::Creative, false),
                0,
                0,
                16,
                "default".to_string(),
            )?;
    
    
            client.set_position(position)?;
    
            let view_chunks = view.chunks();
            for chunk in &view_chunks {
                world.add_player_to_chunk(
                    client,
                    id,
                    ChunkLocation {
                        position: *chunk,
                        location: Location::new(0, 0),
                    },
                )?;
            }
            sync_entities.push((id, (view, Location::new(0, 0))));
    
        }
    }

    let server = state.resources().get::<Server>();
    let world = state.resources().get::<GameWorld>();
    for (id, view) in sync_entities {
        let ecs = state.ecs().borrow();
        state.load_entities_around(&ecs, &server, &world, ecs.entity(id)?, view.1, view.0.chunks().into_iter())?;
    }

    Ok(())
}

pub fn handle_disconnected_clients(state: &GameState) -> anyhow::Result<()> {
    let mut server = state.resources().get_mut::<Server>();
    let mut map = state.resources().get_mut::<ClientMap>();

    let mut to_remove = vec![];
    for cl in server.clients() {
        if cl.is_disconnected() {
            tracing::info!("{} disconnected", cl.profile.name);
            to_remove.push(cl.id);
        }
    }


    let mut ecs = state.ecs().borrow_mut();

    let mut world = state.resources().get_mut::<GameWorld>();
    for cl in to_remove {

        let en = map.get_mapping(cl);
        {
            let entity = ecs.entity(en)?;
            let loc = *entity.get::<&EntityLocation>().unwrap();
    
            let settings = entity.get::<&ClientSettings>().unwrap();

            let chunk_view = View::new(loc.position.chunk(), settings.view_distance as u32);

            {
                let cl = server.get_client(cl)?;
                for chunk in chunk_view.chunks() {
                    world.remove_player_from_chunk(cl, en, ChunkLocation {
                        position: chunk,
                        location: loc.location
                    })?;
                }
            }


    
            let view = View::new(loc.position.chunk(), 8);
            
            state.unload_entities_for(&ecs, &server, &world, entity, loc.location, view.chunks().into_iter())?;
    
            server.remove_client(cl);
            map.remove_mapping(cl);
        }
        ecs.despawn(en)?;
    }
    Ok(())
}
