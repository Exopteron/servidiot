use crate::io::{packet::{def_packets, packet_enum}};



def_packets! {
    LoginStart {
        name: String
    }
}

packet_enum!(ServerPlayPacket {
    LoginStart = 0x00
});