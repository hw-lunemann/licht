mod linear;
mod blend;
mod geometric;
mod parabolic;
mod absolute;

pub use linear::Linear;
pub use blend::Blend;
pub use geometric::Geometric;
pub use parabolic::Parabolic;
pub use absolute::Absolute;

pub trait Stepping {
    fn calculate(&self, step: i32, cur: usize, max: usize) -> f32;
}
