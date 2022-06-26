use super::Stepping;

pub struct Absolute;

impl Stepping for Absolute {
    fn calculate(&self, step: i32, _: usize, _: usize) -> f32 {
        step as f32
    }
}
