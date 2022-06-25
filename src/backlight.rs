use anyhow::Context;
use std::path::{Path, PathBuf};

use crate::Stepping;

pub struct Backlight {
    pub brightness: usize,
    pub brightness_path: PathBuf,
    pub max_brightness: usize,
}

impl Backlight {
    fn read_to_usize<P: AsRef<Path>>(path: P) -> anyhow::Result<usize> {
        let text = std::fs::read_to_string(&path)?;
        text.replace('\n', "").parse().context("parse failure")
    }

    pub fn new(name: &str) -> anyhow::Result<Self> {
        let device_path = Path::new("/sys/class/backlight/").join(name);
        let brightness_path = device_path.join("brightness");

        Ok(Self {
            brightness: Self::read_to_usize(&brightness_path)?,
            max_brightness: Self::read_to_usize(device_path.join("max_brightness"))?,
            brightness_path: device_path.join(brightness_path),
        })
    }

    pub fn get_percent(&self) -> f32 {
        self.brightness as f32 / self.max_brightness as f32
    }

    pub fn calculate_brightness(&mut self, step: i32, stepping: &dyn Stepping, min: usize) {
        let new_brightness = stepping.calculate(step, self.brightness, self.max_brightness);

        let new_brightness = self
            .max_brightness
            .min((new_brightness + 0.5f32) as usize)
            .max(min);
        log::info!(
            "{}% -> {}%",
            (self.get_percent() * 100.0f32).round(),
            (new_brightness as f32 / self.max_brightness as f32 * 100.0f32).round()
        );
        self.brightness = new_brightness
    }

    pub fn write(&self) -> anyhow::Result<()> {
        std::fs::write(
            &self.brightness_path,
            &self.brightness.to_string().as_bytes(),
        )
        .context("writing brightness failed")
    }
}
