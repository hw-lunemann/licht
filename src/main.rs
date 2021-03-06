use clap::Parser;

#[macro_use]
mod verbose;

mod light;
use light::Light;

mod stepping;
use stepping::Stepping;

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    action: Action,

    /// Enable verbose output
    #[clap(value_parser, long, global = true)]
    verbose: bool,
}

#[derive(clap::Subcommand)]
enum Action {
    #[clap(group(
        clap::ArgGroup::new("info-fields")
            .multiple(true)
            .required(true)
            .args(&["name", "class", "brightness", "percent", "max-brightness", "everything", "everything-csv"])
    ))]
    Info {
        #[clap(long)]
        name: bool,
        #[clap(long)]
        class: bool,
        #[clap(long)]
        brightness: bool,
        #[clap(long)]
        percent: bool,
        #[clap(long)]
        max_brightness: bool,
        #[clap(long, exclusive(true))]
        everything: bool,
        #[clap(long, exclusive(true))]
        everything_csv: bool,

        /// The backlight or leds device from sysfs to act on. E.g. intel_backlight
        /// If no device name is supplied and unless any other related flag is set
        /// licht will attempt to discover a backlight device in sysfs.
        #[clap(value_parser, long, display_order = 0)]
        device_names: Option<Vec<String>>,
    },
    /// List availble backlight devices
    List,
    Set {
        #[clap(subcommand)]
        mode: SetMode,

        /// Operate on all backlight devices
        #[clap(long)]
        all: bool,

        /// The backlight or leds device from sysfs to act on. E.g. intel_backlight
        /// If no device name is supplied and unless any other related flag is set
        /// licht will attempt to discover a backlight device in sysfs.
        #[clap(value_parser, long, display_order = 0, global = true)]
        device_names: Option<Vec<String>>,

        /// Clamps the brightness to a minimum value.
        #[clap(value_parser, long, default_value("0"), display_order = 5)]
        min_brightness: usize,

        /// Do not write the new brightness value to the backlight device.
        /// dry-run implies verbose
        #[clap(value_parser, long, display_order = 7)]
        dry_run: bool,
    },
}

#[derive(clap::Subcommand)]
enum SetMode {
    /// Sets the current brightness value to <STEP>%
    Absolute {
        #[clap(flatten)]
        absolute: stepping::Absolute,
    },
    /// Adds <STEP>% to the current brightness value
    Linear {
        #[clap(flatten)]
        linear: stepping::Linear,
    },
    /// Maps the current birghtness value onto the function
    /// ratio*x^a + (1-m) * (1-(1-x)^(1/b) and advances it <STEP>% on that function.
    /// Recommended parameters for this function are ratio = 0.75, a = 1.8, b = 2.2.
    /// The argument for that would be --blend (0.75,1.8,2.2)
    /// Enter the above function into e.g. desmos or geogebra and
    /// change the parameters to your liking.
    Blend {
        #[clap(flatten)]
        blend: stepping::Blend,
    },
    /// Multiplies the current brightness value by <STEP>%
    Geometric {
        #[clap(flatten)]
        geometric: stepping::Geometric,
    },
    /// Maps the current brightness value onto a the parabolic function
    /// x^exponent and advances it <STEP>% on that function.
    Parabolic {
        #[clap(flatten)]
        parabolic: stepping::Parabolic,
    },
}

impl SetMode {
    fn get_stepping(&self) -> &dyn Stepping {
        match &self {
            Self::Absolute { absolute } => absolute,
            Self::Linear { linear } => linear,
            Self::Blend { blend } => blend,
            Self::Geometric { geometric } => geometric,
            Self::Parabolic { parabolic } => parabolic,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        verbose_enable!();
    }

    match cli.action {
        Action::Info {
            name,
            class,
            brightness,
            percent,
            max_brightness,
            everything,
            everything_csv,
            device_names,
        } => {
            let info = |device: Light| {
                if everything_csv {
                    println!(
                        "{},{},{},{:.0}%,{}",
                        device.get_name(),
                        device.get_class(),
                        device.brightness,
                        device.get_percent() * 100.0f32,
                        device.max_brightness
                    );
                } else if everything {
                    println!("{}", device);
                } else if name {
                    println!("{}", device.get_name());
                } else if class {
                    println!("{}", device.get_class());
                } else if brightness {
                    println!("{}", device.brightness);
                } else if percent {
                    println!("{:.0}%", device.get_percent() * 100.0f32);
                } else if max_brightness {
                    println!("{}", device.max_brightness);
                }
            };

            if let Some(device_names) = device_names {
                for device in device_names
                    .into_iter()
                    .map(|device_name| Light::from_name(&device_name))
                {
                    info(device?);
                }
            } else {
                verbose!("No device given, choosing a backlight.");
                info(Light::default()?);
            }
        }
        Action::List => {
            for device in light::discover_all()? {
                println!("{}", device);
            }
        }
        Action::Set {
            mode,
            min_brightness,
            all,
            dry_run,
            device_names,
        } => {
            if dry_run {
                verbose_enable!();
            }

            let mut chosen_devices = Vec::new();

            if all {
                chosen_devices.extend(
                    light::discover_backlights()?
                        .filter(|dev| matches!(dev.class, light::DeviceClass::Backlight)),
                );
                if chosen_devices.is_empty() {
                    anyhow::bail!("No backlight devices were found!")
                }
            } else if let Some(device_name) = device_names {
                for device in device_name
                    .into_iter()
                    .map(|device_name| Light::from_name(&device_name))
                {
                    chosen_devices.push(device?);
                }
            } else {
                verbose!("No device given, choosing a backlight.");
                chosen_devices.push(Light::default()?);
            }

            for mut device in chosen_devices {
                device.calculate_brightness(mode.get_stepping(), min_brightness);
                if dry_run {
                    device.write()?;
                }
            }
        }
    }
    Ok(())
}
