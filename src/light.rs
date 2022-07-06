use anyhow::{anyhow, Context};
use std::path::{Path, PathBuf};

use crate::Stepping;

#[derive(Clone, Copy)]
pub enum DeviceClass {
    Backlight,
    Led,
}

impl DeviceClass {
    const BACKLIGHT_PATH: &'static str = "/sys/class/backlight/";
    const LED_PATH: &'static str = "/sys/class/leds/";

    fn path(&self) -> &'static Path {
        match self {
            DeviceClass::Backlight => Path::new(Self::BACKLIGHT_PATH),
            DeviceClass::Led => Path::new(Self::LED_PATH),
        }
    }

    fn from_path(path: &str) -> anyhow::Result<Self> {
        if path == Self::BACKLIGHT_PATH {
            Ok(Self::Backlight)
        } else if path == Self::LED_PATH {
            Ok(Self::Led)
        } else {
            Err(anyhow!("Path is not a sysfs class path"))
        }
    }
}

pub struct Lights {
    pub devices: Vec<Light>,
}

impl Lights {
    pub fn discover_all() -> anyhow::Result<Self> {
        let mut devices = Vec::new();
        if let Ok(backlights) = Self::discover(DeviceClass::Backlight) {
            devices.extend(backlights);
        }
        if let Ok(leds) = Self::discover(DeviceClass::Led) {
            devices.extend(leds);
        }

        if devices.is_empty() {
            anyhow::bail!("Couldn't find any backlight or led devices.")
        } else {
            Ok(Self { devices })
        }
    }

    pub fn discover_backlights() -> anyhow::Result<Self> {
        let mut devices = Vec::new();
        if let Ok(backlights) = Self::discover(DeviceClass::Backlight) {
            devices.extend(backlights);
        }

        if devices.is_empty() {
            anyhow::bail!("Couldn't find any backlight.")
        } else {
            Ok(Self { devices })
        }
    }

    fn discover(class: DeviceClass) -> anyhow::Result<impl Iterator<Item = Light>> {
        let devices = class
            .path()
            .read_dir()
            .map(|read_dir| {
                read_dir
                    .flatten()
                    .filter_map(|dir_entry| Light::from_path(&dir_entry.path()).ok())
            })
            .context("Couldn't read sysfs");

        devices
    }
}

#[derive(Clone)]
pub struct Light {
    pub brightness: usize,
    pub max_brightness: usize,
    pub device_path: PathBuf,
    pub class: DeviceClass,
}

impl Light {
    fn read_to_usize<P: AsRef<Path>>(path: P) -> anyhow::Result<usize> {
        let text = std::fs::read_to_string(&path)?;
        text.replace('\n', "").parse().context("parse failure")
    }

    pub fn default() -> anyhow::Result<Self> {
        let device_path = Path::new(DeviceClass::BACKLIGHT_PATH)
            .read_dir()
            .context(format!(
                "Failed to enumerate devices in \'{}\'",
                DeviceClass::BACKLIGHT_PATH
            ))?
            .into_iter()
            .find_map(|entry| entry.ok())
            .context("Couldn't find any backlight devices")?
            .path();

        Self::from_path(&device_path)
    }

    pub fn from_path(device_path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            brightness: Self::read_to_usize(device_path.join("brightness"))?,
            max_brightness: Self::read_to_usize(device_path.join("max_brightness"))?,
            device_path: device_path.to_owned(),
            class: DeviceClass::from_path(&device_path.to_string_lossy())?,
        })
    }

    pub fn from_name(device_name: &str) -> anyhow::Result<Self> {
        Self::from_path(&Path::new(DeviceClass::BACKLIGHT_PATH).join(device_name))
            .or_else(|_| Self::from_path(&Path::new(DeviceClass::LED_PATH).join(device_name)))
            .context(format!(
                "Couldn't find device with name \'{}\'",
                device_name
            ))
    }

    pub fn get_percent(&self) -> f32 {
        self.brightness as f32 / self.max_brightness as f32
    }

    pub fn get_class(&self) -> &str {
        self.device_path
            .parent()
            .expect("Device_path without class directory")
            .file_name()
            .expect("Device_path without class name")
            .to_str()
            .expect("Invalid class name")
    }

    pub fn get_name(&self) -> &str {
        self.device_path
            .file_name()
            .expect("Bug: device_path without directory name")
            .to_str()
            .expect("Invalid device name")
    }

    pub fn calculate_brightness(&mut self, stepping: &dyn Stepping, min: usize) {
        let new_brightness = stepping
            .calculate(self.brightness, self.max_brightness)
            .clamp(min as f32, self.max_brightness as f32);

        verbose!("{}", self);
        verbose!(
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
}

impl std::fmt::Display for Light {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Device: {}\nCurrent brightness: {} ({:.0}%)\nMax brightness: {}",
            self.device_path.display(),
            self.brightness,
            self.get_percent() * 100.0f32,
            self.max_brightness
        )
    }
}
