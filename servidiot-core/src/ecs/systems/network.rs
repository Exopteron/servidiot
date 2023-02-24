use servidiot_network::server::Server;
use servidiot_primitives::{position::Position, player::{Gamemode, GamemodeType}};

use crate::{ecs::{system::{SystemExecutor, SysResult}, entities::player::{Name, Player}}, MinecraftServer};

pub fn register_systems(s: &mut SystemExecutor<MinecraftServer>) {

    s.group::<Server>().add_system(accept_new_connections);
}

pub fn accept_new_connections(game: &mut MinecraftServer, server: &mut Server) -> SysResult {
    
    for v in server.accept_clients() {
        
        let cl = server.get_client(v)?;
        let gamemode = Gamemode::new(GamemodeType::Creative, false);
        let position = Position::new(10.0, 10.0, 10.0, 1.0, 1.0, false);
        game.ecs.spawn((Player, v, Name(cl.profile.name.clone()), position, gamemode));
        log::info!("Sending join game");
        cl.join_game(gamemode, 0, 0, 10, "flat".to_string())?;
        log::info!("Sending position: {:?}", position);
        cl.set_position(position)?;
    }
    Ok(())
}
