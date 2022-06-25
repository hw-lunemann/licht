use super::Stepping;
use regex::Regex;

#[derive(clap::Args, Clone)]
pub struct Blend {
    #[clap(value_parser, long, default_value("2"))]
    pub ratio: f32,
    #[clap(value_parser, long, default_value("2"))]
    pub a: f32,
    #[clap(value_parser, long, default_value("2"))]
    pub b: f32,
}

impl Stepping for Blend {
    fn calculate(&self, step: i32, cur: usize, max: usize) -> f32 {
        let step = step as f32 / 100.0f32;
        let f = |x: f32| x.powf(self.a);
        let f_inverse = |x: f32| x.powf(self.a.recip());
        let g = |x: f32| 1.0f32 - (1.0f32 - x).powf(self.b.recip());
        let g_inverse = |x: f32| 1.0f32 - (1.0f32 - x).powf(self.b);
        let h = |x: f32| max as f32 * (self.ratio * f(x) + (1.0f32 - self.ratio) * g(x));

        let cur_f_inv = f_inverse(cur as f32 / max as f32);
        let cur_g_inv = g_inverse(cur as f32 / max as f32);
        let mut l = cur_f_inv.min(cur_g_inv);
        let mut r = cur_f_inv.max(cur_g_inv);

        let first_guess = self.ratio * l + (1.0f32 - self.ratio) * r;
        let mut cur_x = first_guess;

        loop {
            let diff = h(cur_x) - cur as f32;

            if diff.abs() <= max as f32 * 0.001f32 {
                break;
            }

            if diff > 0.0f32 {
                r = cur_x;
            } else {
                l = cur_x;
            }

            cur_x = (l + (r - l) / 2.0f32).clamp(0.0f32, 1.0f32);
        }

        let new_x = (cur_x + step).clamp(0.0f32, 1.0f32);
        h(new_x)
    }
}

impl std::str::FromStr for Blend {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"\(.*,.*,.*\)").unwrap();
        if !regex.is_match(s) {
            anyhow::bail!("Blend parameters malformed")
        }

        let s = &s[1..s.len() - 1];
        let nums: Vec<&str> = s.split(',').collect();
        if nums.len() != 3 {
            anyhow::bail!("Blend parameters malformed: too many paramters")
        }

        let ratio = nums[0].parse::<f32>()?;
        let a = nums[1].parse::<f32>()?;
        let b = nums[2].parse::<f32>()?;

        Ok(Self { ratio, a, b })
    }
}
