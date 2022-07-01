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
        // these checks save 5% when we don't return from them!
        if cur == max && self.step > 0 {
            return max as f32;
        } else if cur == 0 && self.step < 0 {
            return 0.0f32;
        }

        let max = max as i32;
        let cur = cur as i32;

        let f = |x: f32| max as f32 * x.powf(self.a);
        let f_inverse = |y: i32| (y as f32/max as f32).powf(self.a.recip());
        let g = |x: f32| max as f32 * (1.0f32 - (1.0f32 - x).powf(self.b.recip()));
        let g_inverse = |y: i32| 1.0f32 - (1.0f32 - (y as f32/max as f32)).powf(self.b);
        let _h = |x: f32| {
            ((self.ratio*f(x) + (1.0f32-self.ratio)*g(x)) + 0.5f32) as i32
        };

        // h is bounded by l and r
        let mut l = f_inverse(cur).min(g_inverse(cur));
        let mut r = f_inverse(cur).max(g_inverse(cur));
        // this is often a good guess for high x
        let mut cur_x = self.ratio * f_inverse(cur) + (1.0f32 - self.ratio) * g_inverse(cur);

        // this runs faster because simd
        let h = |x: f32| {
            ((max as f32 * self.ratio * x.powf(self.a) 
              - max as f32 * (1.0f32 - self.ratio) * (1.0f32-x).powf(self.b.recip())) + 0.5f32) as i32
        };
        // moving the second term from h to here is 5% performance right there
        let cur = cur - (max as f32 * (1.0f32 - self.ratio)) as i32;

        loop {
            // Integer math is faster here than fp math
            let diff = h(cur_x) as i32 - cur;
            // even with this requirement
            if diff == 0 {
                break;
            }

            if diff < 0 {
                l = cur_x;
            } else {
                r = cur_x;
            }

            cur_x = l + (r - l)/2.0f32;
        }
        

        let new_x = cur_x + self.step as f32 / 100.0f32;

        // this check lets the compiler optimize away another 4 whole nanoseconds!
        if new_x >= 1.0f32 {
            return max as f32;
        } else if new_x <= 0.0f32 {
            return 0.0f32
        }
        // have to add the term back!
        h(new_x) as f32 + max as f32 * (1.0f32 - self.ratio) 
    }
}
