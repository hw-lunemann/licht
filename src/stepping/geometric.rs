use super::Stepping;

#[derive(clap::Args)]
pub struct Geometric {
    #[clap(allow_hyphen_values(true))]
    step: i32,
}

impl Stepping for Geometric {
    fn calculate(&self, cur: usize, _: usize) -> f32 {
        let step = self.step as f32 / 100.0f32;
        cur as f32 + cur as f32 * step
    }
}
