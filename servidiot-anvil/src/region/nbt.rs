use std::ops::{Deref, DerefMut};

use nbt::Value;
use serde::{Serialize, Deserialize};
use servidiot_primitives::nibble_vec::NibbleVec;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkRoot {
    /// Chunk data. 
    #[serde(rename = "Level")]
    pub level: Level
}

/// Byte array helper type.
#[repr(transparent)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct ByteArray(
    #[serde(serialize_with="nbt::i8_array")]
    pub Vec<i8>
);
impl Deref for ByteArray {
    type Target = Vec<i8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ByteArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ByteArray {
    pub fn as_u8_array(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(self.0.as_slice().as_ptr().cast::<u8>(), self.0.len())
        }
    }
}


/// Int array helper type.
#[repr(transparent)]
#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct IntArray(
    #[serde(serialize_with="nbt::i32_array")]
    pub Vec<i32>
);

impl Deref for IntArray {
    type Target = Vec<i32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for IntArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}



#[derive(Serialize, Deserialize, Debug)]
pub struct Level {
    /// X position of the chunk.
    #[serde(rename = "xPos")]
    pub x_position: i32,
    /// Z position of the chunk.
    #[serde(rename = "zPos")]
    pub z_position: i32,
    /// Tick when the chunk was last saved.
    #[serde(rename = "LastUpdate")]
    pub last_update: i64,
    /// Unknown.
    #[serde(rename = "LightPopulated")]
    pub light_populated: Option<bool>,
    /// Indicates whether the terrain in this chunk has been 
    /// populated with special things. (Ores, special blocks, 
    /// trees, dungeons, flowers, waterfalls, etc.) 
    #[serde(rename = "TerrainPopulated")]
    pub terrain_populated: bool,
    /// Likely a chunk version tag.
    #[serde(rename = "V")]
    pub version: Option<i8>,
    /// The cumulative number of ticks 
    /// players have been in this chunk.
    #[serde(rename = "InhabitedTime")]
    pub inhabited_time: i64,
    /// 256 bytes of biome data, one byte for 
    /// each vertical column in the chunk.
    #[serde(rename = "Biomes")]
    pub biomes: Option<ByteArray>,
    /// 16 × 16 heightmap data. 
    #[serde(rename = "HeightMap")]
    pub heightmap: IntArray,
    /// List of sections in this chunk.
    #[serde(rename = "Sections")]
    pub sections: Vec<Section>,
    /// Each TAG_Compound in this list 
    /// defines an entity in the chunk.
    #[serde(rename = "Entities")]
    pub entities: Vec<Value>,
    /// Each TAG_Compound in this list 
    /// defines a tile entity in the 
    /// chunk.
    #[serde(rename = "TileEntities")]
    pub tile_entities: Vec<Value>,
    /// Each TAG_Compound in this list is an 
    /// "active" block in this chunk waiting 
    /// to be updated. These are used to save 
    /// the state of redstone machines, falling 
    /// sand or water, and other activity. 
    #[serde(rename = "TileTicks")]
    pub tile_ticks: Option<Vec<TileTick>>

}

#[derive(Serialize, Deserialize, Debug)]
pub struct Section {
    /// The Y index (not coordinate) of this 
    /// section. Range 0 to 15 (bottom to top), 
    /// with no duplicates but some sections 
    /// may be missing if empty.
    #[serde(rename = "Y")]
    pub y_index: i8,
    /// 4096 bytes of block IDs defining the terrain. 
    /// 8 bits per block, plus the bits from the 
    /// below Add tag.
    #[serde(rename = "Blocks")]
    pub blocks: ByteArray,
    /// May not exist. 2048 bytes of additional block 
    /// ID data. The value to add to (combine with) 
    /// the above block ID to form the true block ID 
    /// in the range 0 to 4095. 4 bits per block. 
    #[serde(rename = "Add")]
    
    pub additional: Option<NibbleVec>,
    /// 2048 bytes of block data additionally defining 
    /// parts of the terrain. 4 bits per block.
    #[serde(rename = "Data")]
    pub data: NibbleVec,
    /// 2048 bytes recording the amount of block-emitted 
    /// light in each block. 4 bits per block.
    #[serde(rename = "BlockLight")]
    pub block_light: NibbleVec,
    /// 2048 bytes recording the amount of sunlight or 
    /// moonlight hitting each block. 4 bits per block.
    #[serde(rename = "SkyLight")]
    pub sky_light: NibbleVec
}

/// Tile Ticks represent block updates that need to happen 
/// because they could not happen before the chunk was saved. 
/// Examples reasons for tile ticks include redstone circuits 
/// needing to continue updating, water and lava that should 
/// continue flowing, recently placed sand or gravel that 
/// should fall, etc. 
#[derive(Serialize, Deserialize, Debug)]
pub struct TileTick {
    /// The ID of the block as an integer.
    #[serde(rename = "i")]
    pub block_id: i32,
    /// The number of ticks until processing 
    /// should occur. May be negative when 
    /// processing is overdue.
    #[serde(rename = "t")]
    pub ticks_until: i32,
    /// If multiple tile ticks are scheduled 
    /// for the same tick, tile ticks with lower 
    /// ordering will be processed first. If they also 
    /// have the same ordering, the order is unspecified.
    #[serde(rename = "p")]
    pub ordering: i32,
    /// X position
    pub x: i32,
    /// Y position
    pub y: i32,
    /// Z position
    pub z: i32
}

