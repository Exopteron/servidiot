use servidiot_network::{server::{Server, id::NetworkID}, io::packet::server::play::ChunkBitmap};
use servidiot_primitives::{position::{Position, ChunkPosition}, player::{Gamemode, GamemodeType}};

use crate::{ecs::{system::{SystemExecutor, SysResult}, entities::player::{Name, Player}}, MinecraftServer, world::World};

pub fn register_systems(s: &mut SystemExecutor<MinecraftServer>) {

    s.group::<Server>().add_system(accept_new_connections).add_system(send_keepalives);
}

pub fn send_keepalives(game: &mut MinecraftServer, server: &mut Server) -> SysResult {
    let mut to_remove = vec![];
    for v in server.clients() {
        v.send_keepalive(1)?;
        if v.is_disconnected() {
            log::info!("Disconnect");
            to_remove.push(v.id);
        }
    }
    for id in to_remove {
        let en = game.entity_for(id).ok_or(anyhow::anyhow!("not present"))?;
        server.remove_client(id);
        game.ecs.defer_despawn(en);
    }
    Ok(())
}


pub fn accept_new_connections(game: &mut MinecraftServer, server: &mut Server) -> SysResult {
    
    for v in server.accept_clients() {
        
        let cl = server.get_client(v)?;
        let gamemode = Gamemode::new(GamemodeType::Creative, false);
        let position = Position::new(10.0, 100.0, 10.0, 1.0, 1.0, false);
        game.ecs.spawn((Player, v, Name(cl.profile.name.clone()), position, gamemode));
        log::info!("Sending join game");
        cl.join_game(gamemode, 0, 0, 10, "flat".to_string())?;

        let handle = game.resources.get_mut::<World>()?.dimension_handle(0)?;

        log::info!("Sending chunks");
        //cl.send_chunk(&*handle.get_chunk(ChunkPosition::new(0, 0))?.chunk()?, ChunkBitmap::full())?;
        
        let mut v = vec![];
        for x in -4..4 {
            for z in -4..4 {
                v.push((ChunkPosition::new(x, z), ChunkBitmap::full()));
            }
        }
        // v.push((ChunkPosition::new(0, 0), ChunkBitmap::full()));
        // v.push((ChunkPosition::new(0, 1), ChunkBitmap::full()));
        cl.send_chunks(&handle, true, &v)?;

        log::info!("Sending position: {:?}", position);
        cl.set_position(position)?;
    }
    Ok(())
}
