use super::Stepping;

#[derive(clap::Args, Clone)]
pub struct Absolute;

impl std::str::FromStr for Absolute {
    type Err = anyhow::Error;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Self)
    }
}

impl Stepping for Absolute {
    fn calculate(&self, step: i32, cur: usize, _: usize) -> f32 {
        cur as f32 + step as f32
    }
}
