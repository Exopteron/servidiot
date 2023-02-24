use crate::io::{packet::{def_packets, packet_enum}, LengthPrefixedVec};


def_packets! {
    LoginStart {
        name: String
    },
    EncryptionResponse {
        shared_secret: LengthPrefixedVec<i16, u8>,
        verify_token: LengthPrefixedVec<i16, u8>
    }
}

packet_enum!(ClientLoginPacket {
    LoginStart = 0x00,
    EncryptionResponse = 0x01
});