use std::fmt::Debug;
use fixed::FixedU32;

#[derive(Debug, Clone)]
pub struct RotationFraction360(pub f32);
impl From<f32> for RotationFraction360 {
    fn from(input: f32) -> Self {
        Self(input)
    }
}

pub type FixedPoint = FixedU32<fixed::types::extra::U5>;
