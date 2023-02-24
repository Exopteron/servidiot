use crate::io::{packet::{def_packets, def_user_enum, packet_enum}, VarInt};

def_user_enum!(NextState (VarInt) {
    Status = 1,
    Login = 2
});

def_packets! {
    Handshake {
        protocol_version: VarInt,
        server_address: String,
        server_port: u16,
        next_state: NextState
    }
}

packet_enum!(ClientHandshakePacket {
    Handshake = 0x00
});