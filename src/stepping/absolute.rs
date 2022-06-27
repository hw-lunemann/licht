use super::Stepping;

#[derive(clap::Args)]
pub struct Absolute {
    value: usize,
}

impl Stepping for Absolute {
    fn calculate(&self, _: usize, _: usize) -> f32 {
        self.value as f32
    }
}
