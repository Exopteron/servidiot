use servidiot_primitives::player::Gamemode;

use crate::io::{packet::{def_packets, packet_enum}};



def_packets! {
    PlayerPositionAndLook {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool
    },
    JoinGame {
        entity_id: i32,
        gamemode: Gamemode,
        dimension: i8,
        difficulty: u8,
        max_players: u8,
        level_type: String
    }
}

packet_enum!(ServerPlayPacket {
    PlayerPositionAndLook = 0x08,
    JoinGame = 0x01
});