use super::Stepping;

pub struct Geometric;

impl Stepping for Geometric {
    fn calculate(&self, step: i32, cur: usize, _: usize) -> f32 {
        let step = step as f32 / 100.0f32;
        cur as f32 + cur as f32 * step
    }
}
