mod absolute;
mod geometric;
mod parabolic;
mod blend;

trait Stepping {
    fn calculate(&self, step: i32, cur: usize, max: usize) -> f32;
}
