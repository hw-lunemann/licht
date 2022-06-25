use super::Stepping;
use regex::Regex;

#[derive(clap::Args, Clone)]
pub struct Parabolic {
    pub exponent: f32,
}

impl Stepping for Parabolic {
    fn calculate(&self, step: i32, cur: usize, max: usize) -> f32 {
        let cur_x = (cur as f32 / max as f32).powf(self.exponent.recip());
        let new_x = cur_x + (step as f32 / 100.0f32);

        max as f32 * new_x.powf(self.exponent)
    }
}

impl std::str::FromStr for Parabolic {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"\(.*\)").unwrap();
        if !regex.is_match(s) {
            anyhow::bail!("Parabolic parameters malformed")
        }

        let s = &s[1..s.len() - 1];
        if s.len() < 3 {
            anyhow::bail!("Parabolic parameters malformed")
        }

        let exponent = s.parse::<f32>()?;

        Ok(Self { exponent })
    }
}
