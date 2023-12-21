use super::BlockPosition;



pub trait BoundedStep: Copy {
    fn steps_between(&self, end: &Self) -> usize;

    fn advanced_to(&self, end: &Self, steps: usize) -> Self;
}

/// Represents an iterator over block positions in a cube.
pub struct BlockPositionIterator {
    start: BlockPosition,
    end: BlockPosition,
    current: BlockPosition,
}

impl BoundedStep for BlockPosition {
    fn steps_between(&self, end: &Self) -> usize {
        (self.x.abs_diff(end.x).max(1) * self.y.abs_diff(end.y).max(1) * self.z.abs_diff(end.z).max(1)) as usize
    }

    fn advanced_to(&self, end: &Self, steps: usize) -> Self {
        let steps = steps as i32;
        let mut new = *self;
        

        let end = end.offset(1, 1, 1);

        new.x = self.x + (steps % end.x);
        new.y = self.y + (steps / end.x) % end.y;
        new.z = self.z + ((steps / end.y) / end.z) % end.z;

        new
    }

}


#[cfg(test)]
mod tests {
    use crate::position::BlockPosition;

    use super::BoundedStep;

    #[test]
    fn iter_test() {
        let start = BlockPosition::new(0, 0, 0);
        let end = BlockPosition::new(0, 0, 3);

        for i in 0..start.steps_between(&end) {
            println!("Val: {}", start.advanced_to(&end, i));
        }   
    }
}