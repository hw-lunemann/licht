pub mod linear;
pub mod blend;
pub mod geometric;
pub mod parabolic;
pub mod set;

pub use linear::Linear;
pub use blend::Blend;
pub use geometric::Geometric;
pub use parabolic::Parabolic;
pub use set::Set;

pub trait Stepping {
    fn calculate(&self, step: i32, cur: usize, max: usize) -> f32;
}
