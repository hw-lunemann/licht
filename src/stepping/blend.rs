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
    #[inline]
    fn calculate(&self, cur: usize, max: usize) -> f32 {
        if cur == max && self.step > 0 {
            return max as f32;
        } else if cur == 0 && self.step < 0 {
            return 0.0f32;
        }

        let f = |x: f32| x.powf(self.a);
        let f_inverse = |y: usize| (y as f32/max as f32).powf(self.a.recip());
        let g = |x: f32| 1.0f32 - (1.0f32 - x).powf(self.b.recip());
        let g_inverse = |y: usize| 1.0f32 - (1.0f32 - (y as f32/max as f32)).powf(self.b);
        let h = |x: f32| {
            self.ratio*f(x) + (1.0f32-self.ratio)*g(x)
        };

        let h_dash = |x: f32| {
            self.a*self.ratio*x.powf(self.a - 1.0f32) - ((self.ratio - 1.0f32)*(1.0f32 - x).powf(self.b.recip() - 1.0f32))/self.b
        };

        let mut cur_x = self.ratio * f_inverse(cur) + (1.0f32 - self.ratio) * g_inverse(cur);

        while (h(cur_x) * max as f32 - cur as f32) as i32 != 0 {
            // newton's method
            cur_x = cur_x - (h(cur_x) - cur as f32/max as f32)/h_dash(cur_x);
        }
        
        let new_x = cur_x + self.step as f32 / 100.0f32;

        if new_x >= 1.0f32 {
            return max as f32;
        } else if new_x <= 0.0f32 {
            return 0.0f32
        }

        h(new_x) * max as f32
    }
}
