use crate::io::{packet::{def_packets, packet_enum}, LengthPrefixedVec};


def_packets! {
    EncryptionRequest {
        server_id: String,
        public_key: LengthPrefixedVec<i16, u8>,
        verify_token: LengthPrefixedVec<i16, u8>
    },
    LoginSuccess {
        uuid: String,
        username: String
    },
    Disconnect {
        data: String
    }
}

packet_enum!(ServerLoginPacket {
    Disconnect = 0x00,
    EncryptionRequest = 0x01,
    LoginSuccess = 0x02
});