use super::Stepping;

#[derive(clap::Args)]
pub struct Blend {
    pub ratio: f32,
    pub a: f32,
    pub b: f32,
    #[clap(allow_hyphen_values(true))]
    pub step: i32,
}

impl Stepping for Blend {
    fn calculate(&self, cur: usize, max: usize) -> f32 {
        let step = self.step as f32 / 100.0f32;
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
