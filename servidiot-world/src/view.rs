use std::collections::HashSet;

use servidiot_primitives::position::{ChunkLocation, ChunkPosition};

pub struct View {
    center: ChunkLocation,
    radius: u32
}
impl View {
    pub fn new(center: ChunkLocation, radius: u32) -> Self {
        Self {
            center,
            radius
        }
    }

    pub fn chunks(&self) -> HashSet<ChunkLocation> {
        let mut set = HashSet::new();
        let center = self.center.position;
        for x in (center.x - self.radius as i32)..(center.x + self.radius as i32) {
            for z in (center.z - self.radius as i32)..(center.z + self.radius as i32) {
                set.insert(ChunkLocation::new(ChunkPosition::new(x, z), self.center.location));
            }
        }
        set
    }

    pub fn contains(&self, c: ChunkLocation) -> bool {
        let c = c.position;
        let center = self.center.position;
        c.x > (center.x - self.radius as i32) &&
        c.x < (center.x + self.radius as i32) &&
        c.z > (center.z - self.radius as i32) &&
        c.z < (center.z + self.radius as i32)
    }

    pub fn difference(&self, other: &View) -> impl Iterator<Item = ChunkLocation> {
        let self_chunks = self.chunks();
        let other_chunks = other.chunks();
        self_chunks.difference(&other_chunks).copied().collect::<Vec<_>>().into_iter()
    }
}
