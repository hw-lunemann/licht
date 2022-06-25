use anyhow::Context;
use clap::Parser;
use simple_logger::SimpleLogger;
use std::path::{Path, PathBuf};
use regex::Regex;

mod stepping;

#[derive(Parser)]
#[clap(group(clap::ArgGroup::new("stepping-mode").args(&["absolute", "geometric", "parabolic", "blend"]).multiple(false)))]
struct Cli {
    #[clap(value_parser, display_order = 0)]
    /// The backlight class device from sysfs to control. E.g. intel_backlight
    device: String,
    #[clap(value_parser, allow_hyphen_values(true))]
    /// The step used by the chosen stepping. By default it's +-% on the parabolic curve x^2.
    step: i32,
    #[clap(value_parser, long, display_order = 1)]
    /// Simply adds the raw step value onto the raw current brightness value
    absolute: Option<Absolute>,
    #[clap(value_parser, long, display_order = 2)]
    /// Multiplies the current brightness value by <STEP>%
    geometric: Option<Geometric>,
    #[clap(value_parser, long, value_name = "(exponent)", display_order = 3)]
    /// Maps the current brightness value onto a the parabolic function
    /// x^exponent and advances it <STEP>% on that function. 
    parabolic: Option<Parabolic>,
    #[clap(value_parser, long, value_name = "(ratio,a,b)", display_order = 4)]
    /// Maps the current birghtness value onto the function 
    /// ratio*x^a + (1-m) * (1-(1-x)^(1/b) and advances it <STEP>% on that function.
    /// Recommended parameters for this function are ratio = 0.75, a = 1.8, b = 2.2.
    /// The argument for that would be --blend (0.75,1.8,2.2)
    /// Enter the above function into e.g. desmos or geogebra and
    /// change the parameters to your liking.
    blend: Option<Blend>,
    #[clap(value_parser, long, default_value("0"), display_order = 5)] 
    /// Clamps the brightness to a minimum value.
    min_brightness: usize,
    #[clap(value_parser, long, display_order = 6)]
    /// Use verbose output
    verbose: bool,
    #[clap(value_parser, long, display_order = 7)]
    /// Do not write the new brightness value to the backlight device.
    /// dry-run implies verbose
    dry_run: bool,
}

impl Cli {

fn get_stepping(&self) -> Option<&dyn Stepping> {
    (self.absolute.as_ref().map(|s| s as &dyn Stepping))
        .or_else(|| self.geometric.as_ref().map(|s| s as &dyn Stepping))
        .or_else(|| self.parabolic.as_ref().map(|s| s as &dyn Stepping))
        .or_else(|| self.blend.as_ref().map(|s| s as &dyn Stepping))
}
}

trait Stepping {
    fn calculate(&self, step: i32, cur: usize, max: usize) -> f32;
}

#[derive(clap::Args, Clone)]
struct Absolute;

impl std::str::FromStr for Absolute {
    type Err = anyhow::Error;

    fn from_str(_: &str) -> Result<Self, Self::Err> {
        Ok(Self)
    }
}

impl Stepping for Absolute {
    fn calculate(&self, step: i32, cur: usize, _:usize) -> f32 {
        cur as f32 + step as f32
    }
}

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

#[derive(clap::Args, Clone)]
struct Parabolic {
    exponent: f32,
}

impl Stepping for Parabolic {
    fn calculate(&self, step: i32, cur: usize, max:usize) -> f32 {
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

        let s = &s[1..s.len()-1];
        if s.len() < 3 {
            anyhow::bail!("Parabolic parameters malformed")
        }

        let exponent = s.parse::<f32>()?;

        Ok(Self {
            exponent
        })
    }
}

#[derive(clap::Args, Clone)]
struct Blend {
    #[clap(value_parser, long, default_value("2"))]
    ratio: f32,
    #[clap(value_parser, long, default_value("2"))]
    a: f32,
    #[clap(value_parser, long, default_value("2"))]
    b: f32,
}

impl Stepping for Blend {
    fn calculate(&self, step: i32, cur: usize, max: usize) -> f32 {
        let step = step as f32 / 100.0f32;
        let f = |x: f32| x.powf(self.a);
        let f_inverse = |x: f32| x.powf(self.a.recip());
        let g = |x: f32| 1.0f32 - (1.0f32 - x).powf(self.b.recip());
        let g_inverse = |x: f32| 1.0f32 - (1.0f32 - x).powf(self.b);
        let h =
            |x: f32| max as f32 * (self.ratio * f(x) + (1.0f32 - self.ratio) * g(x));

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

        let s = &s[1..s.len()-1];
        let nums: Vec<&str> = s.split(',').collect();
        if nums.len() != 3 {
            anyhow::bail!("Blend parameters malformed: too many paramters")
        }

        let ratio = nums[0].parse::<f32>()?;
        let a = nums[1].parse::<f32>()?;
        let b = nums[2].parse::<f32>()?;

        Ok(Self {
            ratio,
            a,
            b
        })
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
            brightness_path: device_path.join(brightness_path),
        })
    }

    fn get_percent(&self) -> f32 {
        self.brightness as f32 / self.max_brightness as f32
    }

    fn calculate_brightness(&mut self, step: i32, stepping: &dyn Stepping, min: usize) {
        let new_brightness = stepping.calculate(step, self.brightness, self.max_brightness);

        let new_brightness = self.max_brightness.min((new_brightness + 0.5f32) as usize).max(min);
        log::info!("{}% -> {}%", (self.get_percent() * 100.0f32).round(), (new_brightness as f32 / self.max_brightness as f32 * 100.0f32).round());
        self.brightness = new_brightness
    }

    fn write(&self) -> anyhow::Result<()> {
        std::fs::write(
            &self.brightness_path,
            &self.brightness.to_string().as_bytes(),
        )
        .context("writing brightness failed")
    }
}

fn read_to_usize<P: AsRef<Path>>(path: P) -> anyhow::Result<usize> {
    let text = std::fs::read_to_string(&path)?;
    text.replace('\n', "").parse().context("parse failure")
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        let logger = SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .without_timestamps()
            .init();
        if logger.is_err() {
            eprint!("Error: logger for verbose mode failed to init.");
        }
    }

    let mut backlight = Backlight::new(&cli.device)?;
    log::info!("Device: {}", cli.device);
    log::info!("Current brightness: {} ({:.0}%)", backlight.brightness, backlight.get_percent()*100.0f32);
    log::info!("Max brightness: {}", backlight.max_brightness);
    backlight.calculate_brightness(cli.step, cli.get_stepping().unwrap_or(&Parabolic { exponent: 2.0f32 }), cli.min_brightness);

    if !cli.dry_run {
        backlight.write()
    } else {
        Ok(())
    }
}
