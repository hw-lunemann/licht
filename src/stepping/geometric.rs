use super::Stepping;

#[derive(clap::Args, Clone)]
struct Geometric; 

impl std::str::FromStr for Geometric {
    type Err = anyhow::Error;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Self)
    }
}

impl Stepping for Geometric {
    fn calculate(&self, step: i32, cur: usize, _: usize) -> f32 {
        let step = step as f32 / 100.0f32;
        cur as f32 + cur as f32 * step
    }
}
