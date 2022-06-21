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
    verbose: bool
}

#[derive(Clone)]
enum Stepping {
    Absolute,
    CurrentRelative,
    Parabolic { 
        exponent: f32
    },
}

impl ValueEnum for Stepping {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Absolute, Self::CurrentRelative, Self::Parabolic { exponent: 2.0f32 }]
    }

    fn to_possible_value<'a>(&self) -> Option<PossibleValue<'a>> {
        match self {
            Self::Absolute => Some(PossibleValue::new("absolute")),
            Self::CurrentRelative => Some(PossibleValue::new("current-relative")),
            Self::Parabolic { .. } => Some(PossibleValue::new("parabolic"))
        }
    }
}

#[derive(Clone)]
enum Action {
    Plus,
    Minus,
}

impl ValueEnum for Action {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Plus, Self::Minus]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::PossibleValue<'a>> {
        match self {
            Self::Plus => Some(PossibleValue::new("+")),
            Self::Minus => Some(PossibleValue::new("-")),
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

    fn change_brightness(&mut self, step: i32, stepping: Stepping) -> anyhow::Result<()> {
        let new_brightness = match stepping {
            Stepping::Absolute => self.brightness as f32 + step as f32,
            Stepping::CurrentRelative => {
                let step = step as f32 / 100.0f32;
                self.brightness as f32 + self.brightness as f32 * step
            }
            Stepping::Parabolic { exponent } => {
                let cur_x = self.get_percent().powf(1.0f32/exponent);
                let new_x = cur_x + (step as f32 / 100.0f32);
                self.max_brightness as f32 * new_x.powf(exponent)
            }
        };

        self.brightness = self.max_brightness.min((new_brightness + 0.5f32)as usize);

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
    if cli.verbose {
        print!("{} -> ", &backlight.get_percent());
    }
    backlight.change_brightness(cli.step, cli.stepping)?;
    if cli.verbose {
        print!("{}", &backlight.get_percent());
    }

    Ok(())
}
