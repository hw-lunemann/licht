mod absolute;
mod blend;
mod geometric;
mod linear;
mod parabolic;

pub use absolute::Absolute;
pub use blend::Blend;
pub use geometric::Geometric;
pub use linear::Linear;
pub use parabolic::Parabolic;

pub trait Stepping {
    fn calculate(&self, cur: usize, max: usize) -> f32;
}
