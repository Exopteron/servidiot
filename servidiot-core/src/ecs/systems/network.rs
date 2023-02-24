use servidiot_network::server::Server;
use servidiot_primitives::{position::{Position, ChunkPosition}, player::{Gamemode, GamemodeType}};

use crate::{ecs::{system::{SystemExecutor, SysResult}, entities::player::{Name, Player}}, MinecraftServer, world::World};

pub fn register_systems(s: &mut SystemExecutor<MinecraftServer>) {

    s.group::<Server>().add_system(accept_new_connections);
}

pub fn accept_new_connections(game: &mut MinecraftServer, server: &mut Server) -> SysResult {
    
    for v in server.accept_clients() {
        
        let cl = server.get_client(v)?;
        let gamemode = Gamemode::new(GamemodeType::Creative, false);
        let position = Position::new(10.0, 255.0, 10.0, 1.0, 1.0, false);
        game.ecs.spawn((Player, v, Name(cl.profile.name.clone()), position, gamemode));
        log::info!("Sending join game");
        cl.join_game(gamemode, 0, 0, 10, "flat".to_string())?;

        let handle = game.resources.get_mut::<World>()?.dimension_handle(0)?.get_chunk(ChunkPosition::new(0, 0)).unwrap();

        log::info!("Sending chunk");
        cl.send_chunk(&handle.chunk().unwrap())?;

        log::info!("Sending position: {:?}", position);
        cl.set_position(position)?;
    }
    Ok(())
}
