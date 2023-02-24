use crate::io::{packet::{def_packets, packet_enum}};



def_packets! {
    LoginStart {
        name: String
    }
}

packet_enum!(ClientPlayPacket {
    LoginStart = 0x00
});