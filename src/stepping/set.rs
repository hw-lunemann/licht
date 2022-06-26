use super::Stepping;

pub struct Set;

impl Stepping for Set {
    fn calculate(&self, step: i32, _: usize, _: usize) -> f32 {
        step as f32
    }
}
