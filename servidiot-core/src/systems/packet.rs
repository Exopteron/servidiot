use servidiot_ecs::{EntityRef, SystemExecutor};
use servidiot_network::{
    io::packet::client::play::{self, ClientPlayPacket, ClientSettings},
    server::{Client, Server},
};
use servidiot_primitives::position::{EntityLocation, Position};

use crate::{game::{ClientMap, GameState}, events::entity::{EntityMoveEvent, PlayerViewChangeEvent}, world::view::View};

pub fn register_systems(s: &mut SystemExecutor<GameState>) {
    s.add_system(handle_packets);
}

pub fn handle_packets(state: &GameState) -> anyhow::Result<()> {
    let server = state.resources().get::<Server>();
    let map = state.resources().get::<ClientMap>();

    for client in server.clients() {
        let entity = map.get_mapping(client.id);
        for packet in client.packets() {
            let ecs = state.ecs().borrow();
            let player_entity = ecs.entity(entity)?;
            match packet {
                ClientPlayPacket::Player(p) => {
                    let mut loc = player_entity.get::<&mut EntityLocation>().unwrap();
                    let pos = loc.position;
                    loc.position.on_ground = p.on_ground;
                    handle_new_position(state, client, player_entity, pos, loc.position)?;
                }
                ClientPlayPacket::PlayerPosition(p) => {
                    let mut loc = player_entity.get::<&mut EntityLocation>().unwrap();
                    let pos = loc.position;
                    loc.position.on_ground = p.on_ground;
                    loc.position.x = p.x;
                    loc.position.y = p.feet_y;
                    loc.position.z = p.z;
                    handle_new_position(state, client, player_entity, pos, loc.position)?;
                }
                ClientPlayPacket::PlayerLook(p) => {
                    let mut loc = player_entity.get::<&mut EntityLocation>().unwrap();
                    let pos = loc.position;
                    loc.position.on_ground = p.on_ground;
                    loc.position.yaw = p.yaw;
                    loc.position.pitch = p.pitch;
                    handle_new_position(state, client, player_entity, pos, loc.position)?;
                }
                ClientPlayPacket::PlayerPositionAndLook(p) => {
                    let mut loc = player_entity.get::<&mut EntityLocation>().unwrap();
                    let pos = loc.position;
                    loc.position.on_ground = p.on_ground;
                    loc.position.yaw = p.yaw;
                    loc.position.pitch = p.pitch;
                    loc.position.x = p.x;
                    loc.position.y = p.feet_y;
                    loc.position.z = p.z;
                    handle_new_position(state, client, player_entity, pos, loc.position)?;
                }
                _ => (),
            }
        }
    }

    Ok(())
}

fn handle_new_position(game: &GameState, client: &Client, player: EntityRef, old_pos: Position, new_pos: Position) -> anyhow::Result<()> {
    client.set_client_known_position(new_pos);

    let events = game.events().borrow();
    events.post_event(game, EntityMoveEvent {
        entity: player.entity(),
        old_pos,
        new_pos
    })?;



    if old_pos.chunk() != new_pos.chunk() {
        let settings = player.get::<&ClientSettings>().unwrap();
        events.post_event(game, PlayerViewChangeEvent {
            entity: player.entity(),
            old_view: View::new(old_pos.chunk(), settings.view_distance as u32),
            new_view: View::new(new_pos.chunk(), settings.view_distance as u32)
        })?;
    }

    Ok(())
}
