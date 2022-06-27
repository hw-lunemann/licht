use anyhow::Context;
use std::path::{Path, PathBuf};

use crate::Stepping;

fn class_path() -> &'static Path {
    Path::new("/sys/class/backlight/")
}

pub struct Backlight {
    pub brightness: usize,
    pub max_brightness: usize,
    pub device_path: PathBuf,
}

impl Backlight {
    fn read_to_usize<P: AsRef<Path>>(path: P) -> anyhow::Result<usize> {
        let text = std::fs::read_to_string(&path)?;
        text.replace('\n', "").parse().context("parse failure")
    }

    pub fn from_name(name: &str) -> anyhow::Result<Self> {
        let device_path = class_path().join(name);

        Ok(Self {
            brightness: Self::read_to_usize(device_path.join("brightness"))?,
            max_brightness: Self::read_to_usize(device_path.join("max_brightness"))?,
            device_path
        })
    }

    pub fn from_path(device_path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            brightness: Self::read_to_usize(device_path.join("brightness"))?,
            max_brightness: Self::read_to_usize(device_path.join("max_brightness"))?,
            device_path: device_path.to_owned()
        })
    }

    pub fn get_percent(&self) -> f32 {
        self.brightness as f32 / self.max_brightness as f32
    }

    pub fn calculate_brightness(&mut self, stepping: &dyn Stepping, min: usize) {
        let new_brightness = stepping.calculate(self.brightness, self.max_brightness)
            .clamp(min as f32, self.max_brightness as f32);

        log::info!(
            "{}% -> {}%",
            (self.get_percent() * 100.0f32).round(),
            (new_brightness / self.max_brightness as f32 * 100.0f32).round()
        );
        self.brightness = new_brightness as usize;
    }

    pub fn write(&self) -> anyhow::Result<()> {
        std::fs::write(
            &self.device_path.join("brightness"),
            &self.brightness.to_string().as_bytes(),
        )
        .context("writing brightness failed")
    }

    pub fn discover() -> Vec<PathBuf> {
        let mut devices = Vec::new();
        if let Ok(read_dir) = class_path().read_dir() {
            read_dir.flatten()
                .for_each(|dir_entry| devices.push(dir_entry.path()))
        }

        devices
    }
}

impl std::fmt::Display for Backlight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, 
               "Device: {}\nCurrent brightness: {} ({:.0}%)\nMax brightness: {}", 
               self.device_path.display(), 
               self.brightness, 
               self.get_percent() * 100.0f32, 
               self.max_brightness)
    }
}
