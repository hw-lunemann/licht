use anyhow::Context;
use clap::{Parser, ValueEnum, PossibleValue};
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Cli {
    #[clap(value_parser, display_order = 0)]
    path: PathBuf,
    #[clap(value_enum, display_order = 1)]
    action: Action,
    #[clap(value_parser, display_order = 2)]
    step: usize,
    #[clap(value_enum, long, default_value("relative"))]
    mode: Mode
}

#[derive(clap::ValueEnum, Clone)]
enum Mode {
    CurrentRelative, MaxRelative
}

#[derive(Clone)]
enum Action {
    Plus, Minus
}

impl ValueEnum for Action {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Plus, Self::Minus]
    }

    fn to_possible_value<'a>(&self) -> Option<clap::PossibleValue<'a>> {
        match self {
            Action::Plus => Some(PossibleValue::new("+")),
            Action::Minus => Some(PossibleValue::new("-")),
        }
    }
}

fn calculate_brightness(action: Action, step: usize, mode: Mode, current: f32, max: f32) -> f32 {
    match mode {
        Mode::CurrentRelative => {
            let step = step as f32 / 100.0f32;
            let diff = current * step;
            match action {
                Action::Plus => current + diff,
                Action::Minus => current - diff
            }
        },
        Mode::MaxRelative => { 
            let step = step as f32 / 100.0f32;
            let diff = max * step;
            match action {
                Action::Plus => current + diff,
                Action::Minus => current - diff
            }
        },
    }
}


fn read_to_f32(path: &Path) -> anyhow::Result<f32> {
    let text = std::fs::read_to_string(&path)?;
    text.replace('\n', "").parse().context("parse failure")
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    if !cli.path.starts_with("/sys/class/backlight/") {
        anyhow::bail!("input had to be from the backlight device class in sysfs.")
    }


    let brightness_file = &cli.path.join("brightness");
    let brightness = read_to_f32(brightness_file)?;
    let max_brightness = read_to_f32(&cli.path.join("max_brightness"))?;
    let new_brigtness = calculate_brightness(cli.action, cli.step, cli.mode, brightness, max_brightness)
        .min(max_brightness);

    std::fs::write(brightness_file, new_brigtness.to_string().as_bytes()).context("writing brightness failed")
}
