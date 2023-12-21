use anyhow::bail;
use miniz_oxide::deflate::compress_to_vec_zlib;
use servidiot_primitives::{nibble_vec::NibbleVec, player::Gamemode, position::ChunkPosition, chunk::ChunkBitmap};

use crate::io::{
    packet::{def_packets, packet_enum},
    Readable, Writable,
};

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
        primary_bit_map: ChunkBitmap,
        add_bit_map: ChunkBitmap,
        chunk_data: NetChunk
    },
    MapChunkMeta {
        x: i32,
        z: i32,
        primary_bit_map: ChunkBitmap,
        add_bit_map: ChunkBitmap
    },
    KeepAlive {
        id: i32
    }
}

packet_enum!(ServerPlayPacket {
    KeepAlive = 0x00,
    PlayerPositionAndLook = 0x08,
    JoinGame = 0x01,
    ChunkData = 0x21,
    MapChunkBulk = 0x26
});

#[derive(Debug)]
pub struct MapChunkBulk {
    pub chunk_column_count: i16,
    pub sky_light_sent: bool,
    pub data: Vec<(ChunkPosition, NetChunkData, ChunkBitmap, ChunkBitmap)>,
}
impl Writable for MapChunkBulk {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        self.chunk_column_count.write_to(target)?;
        let mut data_vec = vec![];
        let mut meta_vec = vec![];
        for (pos, v, section_bitmap, add_bitmap) in &self.data {
            if v.block_sky_light.is_none() && self.sky_light_sent {
                bail!("sky light not present in chunk")
            }
            v.write_to(&mut data_vec)?;
            MapChunkMeta {
                x: pos.x,
                z: pos.z,
                primary_bit_map: *section_bitmap,
                add_bit_map: *add_bitmap,
            }.write_to(&mut meta_vec)?;
        }
        let mut data = compress_to_vec_zlib(&data_vec, 5);
        let len: i32 = (data.len()).try_into()?;
        len.write_to(target)?;
        self.sky_light_sent.write_to(target)?;
        target.append(&mut data);
        target.append(&mut meta_vec);

        Ok(())
    }
}

impl Readable for MapChunkBulk {
    fn read_from(_data: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<Self> {
        bail!("op not supported")
    }
}


impl Writable for ChunkBitmap {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        self.0.write_to(target)
    }
}

impl Readable for ChunkBitmap {
    fn read_from(data: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<Self> {
        Ok(Self(u16::read_from(data)?))
    }
}
#[derive(Debug)]
pub struct NetChunkData {
    pub block_types: Vec<u8>,
    pub block_meta: NibbleVec,
    pub block_light: NibbleVec,
    pub block_sky_light: Option<NibbleVec>,
    pub add_array: Option<NibbleVec>,
    pub biome_array: Box<[u8; 256]>,
    pub compressed: bool
}
impl Writable for NetChunkData {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        let mut buf = vec![];
        buf.extend_from_slice(&self.block_types);
        buf.extend_from_slice(self.block_meta.get_backing());
        buf.extend_from_slice(self.block_light.get_backing());
        if let Some(sky_light) = &self.block_sky_light {
            buf.extend_from_slice(sky_light.get_backing());
        }
        if let Some(add_array) = &self.add_array {
            buf.extend_from_slice(add_array.get_backing());
        }
        buf.extend_from_slice(&*self.biome_array);
        if self.compressed {
            let mut compressed = compress_to_vec_zlib(&buf, 5);
            let len: i32 = (compressed.len().try_into())?;
            len.write_to(target)?;
            target.append(&mut compressed);
        } else {
            target.append(&mut buf);
        }
        Ok(())
    }
}


#[derive(Debug)]
pub enum NetChunk {
    Present(NetChunkData),
    NotPresent
}
impl Writable for NetChunk {
    fn write_to(&self, target: &mut Vec<u8>) -> anyhow::Result<()> {
        match self {
            Self::Present(v) => v.write_to(target),
            Self::NotPresent => {
                let mut compressed = compress_to_vec_zlib(&[], 0);
                let len: i32 = (compressed.len().try_into())?;
                len.write_to(target)?;
                target.append(&mut compressed);
                Ok(())
            }
        }
    }
}

impl Readable for NetChunk {
    fn read_from(_data: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<Self> {
        panic!("unsupported")
    }
}
