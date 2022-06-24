use anyhow::Context;
use clap::{Parser, PossibleValue, ValueEnum};
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Cli {
    #[clap(value_parser, display_order = 0)]
    device: String,
    #[clap(value_parser, allow_hyphen_values(true), display_order = 1)]
    step: i32,
    #[clap(value_enum, long, default_value("parabolic"))]
    stepping: Stepping,
    #[clap(value_parser, long, default_value("2"))]
    exponent: f32,
    #[clap(value_parser, long)]
    verbose: bool,
    #[clap(value_parser, long)]
    dry_run: bool

}

#[derive(Clone)]
enum Stepping {
    Absolute,
    Geometric,
    Parabolic { 
        exponent: f32
    },
    Blend {
        ratio: f32,
        a: f32,
        b: f32
    }
}

impl ValueEnum for Stepping {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Absolute, Self::Geometric, Self::Parabolic { exponent: 2.0f32 }, Self::Blend { ratio: 0.75f32, a: 2.2f32, b: 1.8f32 }] 
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue<'a>> {
        match self {
            Self::Absolute => Some(PossibleValue::new("absolute")),
            Self::Geometric => Some(PossibleValue::new("geometric")),
            Self::Parabolic { .. } => Some(PossibleValue::new("parabolic")),
            Self::Blend { .. } => Some(PossibleValue::new("blend"))
        }
    }
}

struct Backlight {
    brightness: usize,
    brightness_path: PathBuf,
    max_brightness: usize,
}

impl Backlight {
    fn new(name: &str) -> anyhow::Result<Self> {
        let device_path = Path::new("/sys/class/backlight/").join(name);
        let brightness_path = device_path.join("brightness");

        Ok(Self {
            brightness: read_to_usize(&brightness_path)?,
            max_brightness: read_to_usize(device_path.join("max_brightness"))?,
            brightness_path: device_path.join(brightness_path)
        })
    }

    fn get_percent(&self) -> f32 {
        self.brightness as f32 / self.max_brightness as f32
    }

    fn calculate_brightness(&mut self, step: i32, stepping: Stepping) {
        let new_brightness = match stepping {
            Stepping::Absolute => self.brightness as f32 + step as f32,
            Stepping::Geometric => {
                let step = step as f32 / 100.0f32;
                self.brightness as f32 + self.brightness as f32 * step
            }
            Stepping::Parabolic { exponent } => {
                let cur_x = self.get_percent().powf(1.0f32/exponent);
                let new_x = cur_x + (step as f32 / 100.0f32);
                
                self.max_brightness as f32 * new_x.powf(exponent)
            }
            Stepping::Blend { ratio, a, b } => {
                let step = step as f32 / 100.0f32;
                let f = |x: f32| x.powf(a);
                let f_inverse = |x: f32| x.powf(a.recip());
                let g = |x: f32| 1.0f32 - (1.0f32 - x).powf(1.0f32/b);
                let g_inverse = |x: f32| 1.0f32 - (1.0f32 - x).powf(b);
                let h = |x: f32| {
                    self.max_brightness as f32 * (ratio * f(x) + (1.0f32-ratio) * g(x))
                };

                let cur_f_inv = f_inverse(self.get_percent());
                let cur_g_inv = g_inverse(self.get_percent());
                let mut l = cur_f_inv.min(cur_g_inv);
                let mut r = cur_f_inv.max(cur_g_inv);

                let first_guess = ratio*l + (1.0f32-ratio)*r;
                let mut cur_x = first_guess;

                loop {
                    let diff = h(cur_x) - self.brightness as f32;
                    
                    if diff.abs() <= self.max_brightness as f32 * 0.001f32 {
                        break
                    }

                    if diff > 0.0f32 {
                        r = cur_x;
                    } else {
                        l = cur_x;
                    }
                    
                    cur_x = (l + (r-l)/2.0f32).clamp(0.0f32, 1.0f32);
                }

                let new_x = (cur_x + step).clamp(0.0f32, 1.0f32);
                h(new_x)
            }
        };

        self.brightness = self.max_brightness.min((new_brightness + 0.5f32)as usize);
    }

    fn write(&self) -> anyhow::Result<()> {
        std::fs::write(&self.brightness_path, &self.brightness.to_string().as_bytes())
            .context("writing brightness failed")
    }
}

fn read_to_usize<P: AsRef<Path>>(path: P) -> anyhow::Result<usize> {
    let text = std::fs::read_to_string(&path)?;
    text.replace('\n', "").parse().context("parse failure")
}

fn main() -> anyhow::Result<()> {
    let mut cli = Cli::parse();
    if let Stepping::Parabolic { .. } = cli.stepping {
        cli.stepping = Stepping::Parabolic { exponent: cli.exponent };
    }

    let mut backlight = Backlight::new(&cli.device)?;
    if cli.verbose || cli.dry_run {
        print!("{} -> ", &backlight.get_percent());
    }
    backlight.calculate_brightness(cli.step, cli.stepping);
    if cli.verbose || cli.dry_run {
        print!("{}", &backlight.get_percent());
    }

    if !cli.dry_run {
        backlight.write()
    } else {
        Ok(())
    }
}
