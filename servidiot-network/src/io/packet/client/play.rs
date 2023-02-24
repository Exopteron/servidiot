use crate::io::{packet::{def_packets, packet_enum}, LengthPrefixedVec};


bitflags::bitflags! {
    struct ChatFlags: i8 {
        const COMMANDS_ONLY = 0b01;
        const HIDDEN = 0b10;
    }
}

def_packets! {
    ClientSettings {
        locale: String,
        view_distance: i8,
        chat_flags: i8,
        chat_colours: bool,
        difficuty: i8,
        show_cape: bool
    },
    PluginMessage {
        channel: String,
        data: LengthPrefixedVec<i16, u8>
    },
    PlayerPositionAndLook {
        x: f64,
        feet_y: f64,
        head_y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool
    },
    PlayerPosition {
        x: f64,
        feet_y: f64,
        head_y: f64,
        z: f64,
        on_ground: bool
    },
    PlayerLook {
        yaw: f32,
        pitch: f32,
        on_ground: bool
    },
    Player {
        on_ground: bool
    },
    PlayerAbilities {
        flags: i8,
        flying_speed: f32,
        walking_speed: f32
    },
    KeepAlive {
        id: i32
    }
}

packet_enum!(ClientPlayPacket {
    ClientSettings = 0x15,
    PluginMessage = 0x17,
    Player = 0x03,
    PlayerPosition = 0x04,
    PlayerLook = 0x05,
    PlayerPositionAndLook = 0x06,
    KeepAlive = 0x00,
    PlayerAbilities = 0x13
});