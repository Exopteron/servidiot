use super::BlockPosition;



pub struct StepIterator<T: BoundedStep> {
    start: T,
    end: T,
    steps: usize,
    steps_between: usize
}
impl<T: BoundedStep> StepIterator<T> {
    pub fn new(start: T, end: T) -> Self {
        Self {
            start,
            end,
            steps: 0,
            steps_between: start.steps_between(&end)
        }
    }
}

impl<T: BoundedStep> Iterator for StepIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.steps >= self.steps_between {
            None
        } else {
            self.steps += 1;
            Some(self.start.advanced_to(&self.end, self.steps - 1))
        }
    }
}

/// Something that can progress some number of steps between a start and an end point.
pub trait BoundedStep: Copy {
    fn steps_between(&self, end: &Self) -> usize;

    fn advanced_to(&self, end: &Self, steps: usize) -> Self;
}

impl BoundedStep for BlockPosition {
    fn steps_between(&self, end: &Self) -> usize {
        (self.x.abs_diff(end.x).wrapping_add(1) * self.y.abs_diff(end.y).wrapping_add(1) * self.z.abs_diff(end.z).wrapping_add(1)) as usize
    }

    fn advanced_to(&self, end: &Self, steps: usize) -> Self {
        let steps = steps as i32;
        let mut new = *self;
        

        let end = end.offset(1, 1, 1);

        new.x = self.x + (steps % end.x);
        new.y = self.y + (steps / end.x) % end.y;
        new.z = self.z + ((steps / end.x) / end.y) % end.z;

        new
    }

}




#[cfg(test)]
mod tests {
    use crate::position::BlockPosition;

    use super::StepIterator;

    #[test]
    fn iter_test() {
        let start = BlockPosition::new(0, 0, 0);
        let end = BlockPosition::new(3, 3, 3);

        
        for i in StepIterator::new(start, end) {
            println!("Val: {}", i);
        }   
    }
}