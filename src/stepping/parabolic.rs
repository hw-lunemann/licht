use super::Stepping;

#[derive(clap::Args)]
pub struct Parabolic {
    pub step: i32,
    pub exponent: f32,
}

impl Stepping for Parabolic {
    fn calculate(&self, cur: usize, max: usize) -> f32 {
        let cur_x = (cur as f32 / max as f32).powf(self.exponent.recip());
        let new_x = cur_x + (self.step as f32 / 100.0f32);

        max as f32 * new_x.powf(self.exponent)
    }
}
