use super::Stepping;

pub struct Absolute;

impl Stepping for Absolute {
    fn calculate(&self, step: i32, cur: usize, _: usize) -> f32 {
        cur as f32 + step as f32
    }
}
