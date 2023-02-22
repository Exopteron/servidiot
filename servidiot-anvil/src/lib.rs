use std::{fs::File, io::{Seek, SeekFrom, Read, self}, collections::LinkedList};

use bitvec::vec::BitVec;
use servidiot_primitives::position::ChunkPosition;

#[repr(u8)]
pub enum CompressionType {
    GZip = 1,
    ZLib = 2,
    Uncompressed = 3
}



pub struct RegionFile {
    chunk_location: [u8; 4096],
    timestamps: [u32; 1024],
    free_sectors: BitVec,
    file: File
}
impl RegionFile {
    pub const BYTES_PER_SECTOR: u64 = 4096;


    pub fn new(mut file: File) -> io::Result<Self> {
        file.rewind()?;
        let mut chunk_location = [0; 4096];
        file.read_exact(&mut chunk_location)?;
        let mut timestamps = [0; 4096];
        file.read_exact(&mut timestamps)?;

        let mut this = Self {
            chunk_location,
            timestamps: unsafe { std::mem::transmute(timestamps) },
            free_sectors: BitVec::new(),
            file
        };
        let file_len = this.file.metadata()?.len();

        for _ in 0..(file_len / Self::BYTES_PER_SECTOR) {
            this.free_sectors.push(true);
        }

        this.free_sectors.set(0, false); // chunk locations
        this.free_sectors.set(1, false); // timestamps

        let mut i = 0;
        while i < this.chunk_location.len() {
            let sum: u32 = this.chunk_location[i..i+4].iter().map(|v| *v as u32).sum();
            if sum != 0 {
                this.free_sectors.set(i / 4, false);
            }
            i += 4;
        }

        Ok(this)
    }   

    fn get_chunk_location(&self, position: ChunkPosition) -> Option<(u32, u8)> {
        let offset = 4 * (position.x & 31) + (position.z & 31) * 32;
        let v = &self.chunk_location[offset as usize..(offset + 4) as usize];
        let offset = u32::from_be_bytes([v[0], v[1], v[2], 0]);
        if offset == 0 && v[3] == 0 {
            None
        } else {
            Some((offset, v[3]))
        }
    }

}