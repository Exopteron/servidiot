use std::collections::HashSet;

use servidiot_primitives::position::ChunkPosition;

pub struct View {
    center: ChunkPosition,
    radius: u32
}
impl View {
    pub fn new(center: ChunkPosition, radius: u32) -> Self {
        Self {
            center,
            radius
        }
    }

    pub fn chunks(&self) -> HashSet<ChunkPosition> {
        let mut set = HashSet::new();
        for x in (self.center.x - self.radius as i32)..(self.center.x + self.radius as i32) {
            for z in (self.center.z - self.radius as i32)..(self.center.z + self.radius as i32) {
                set.insert(ChunkPosition::new(x, z));
            }
        }
        set
    }

    pub fn contains(&self, c: ChunkPosition) -> bool {
        c.x > (self.center.x - self.radius as i32) &&
        c.x < (self.center.x + self.radius as i32) &&
        c.z > (self.center.z - self.radius as i32) &&
        c.z < (self.center.z + self.radius as i32)
    }
}