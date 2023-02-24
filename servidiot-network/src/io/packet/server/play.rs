use miniz_oxide::deflate::compress_to_vec_zlib;
use servidiot_primitives::{player::Gamemode, nibble_vec::NibbleVec};

use crate::io::{packet::{def_packets, packet_enum}, Writable, Readable};



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
    },
    ChunkData {
        chunk_x: i32,
        chunk_z: i32,
        ground_up_continuous: bool,
        primary_bit_map: u16,
        add_bit_map: u16,
        chunk_data: NetChunk
    }
}

packet_enum!(ServerPlayPacket {
    PlayerPositionAndLook = 0x08,
    JoinGame = 0x01,
    ChunkData = 0x21
});

#[derive(Debug)]
pub struct NetChunk {
    pub block_types: Vec<u8>,
    pub block_meta: NibbleVec,
    pub block_light: NibbleVec,
    pub block_sky_light: Option<NibbleVec>,
    pub add_array: NibbleVec,
    pub biome_array: Box<[u8; 256]>
}
impl Writable for NetChunk {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        let mut buf = vec![];
        buf.extend_from_slice(&self.block_types);
        buf.extend_from_slice(self.block_meta.get_backing());
        if let Some(sky_light) = &self.block_sky_light {
            buf.extend_from_slice(sky_light.get_backing());
        }
        buf.extend_from_slice(self.add_array.get_backing());
        buf.extend_from_slice(&*self.biome_array);
        let mut compressed = compress_to_vec_zlib(&buf, 5);
        let len: i32 = (compressed.len().try_into())?;
        len.write_to(target)?;
        target.append(&mut compressed);
        Ok(())
    }
}

impl Readable for NetChunk {
    fn read_from(_data: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<Self> {
        panic!("unsupported")
    }
}