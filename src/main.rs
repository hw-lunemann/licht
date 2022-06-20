use anyhow::Context;
use clap::Parser;
use std::path::{Path, PathBuf};

#[derive(Parser)]
struct Cli {
    #[clap(value_parser, display_order = 0)]
    path: PathBuf,
    #[clap(value_parser, possible_values(&["+", "-"]), display_order = 1)]
    action: char,
    #[clap(value_parser, display_order = 2)]
    step: usize,
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


    let factor = match cli.action {
        '-' => 1.0 - cli.step as f32 / 100.0,
        '+' => 1.0 + cli.step as f32 / 100.0,
        _ => unreachable!(),
    };
    let brightness_file = &cli.path.join("brightness");
    let brightness = read_to_f32(brightness_file)?;
    let max_brightness = read_to_f32(&cli.path.join("max_brightness"))?;
    let min = max_brightness.min(brightness * factor) as usize;

    std::fs::write(brightness_file, min.to_string().as_bytes()).context("writing brightness failed")
}
