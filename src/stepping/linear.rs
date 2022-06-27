use super::Stepping;

#[derive(clap::Args)]
pub struct Linear {
    step: i32
}

impl Stepping for Linear {
    fn calculate(&self, cur: usize, _: usize) -> f32 {
        cur as f32 + self.step as f32
    }
}
