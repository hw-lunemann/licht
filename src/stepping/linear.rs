use super::Stepping;

pub struct Linear;

impl Stepping for Linear {
    fn calculate(&self, step: i32, cur: usize, _: usize) -> f32 {
        cur as f32 + step as f32
    }
}
