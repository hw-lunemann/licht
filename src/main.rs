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
}

#[derive(clap::Subcommand)]
enum Action {
    Get {
        #[clap(subcommand)]
        mode: GetMode,
    },
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
        device_name: Option<String>,

        /// Clamps the brightness to a minimum value.
        #[clap(value_parser, long, default_value("0"), display_order = 5)]
        min_brightness: usize,

        /// Use verbose output
        #[clap(value_parser, long, display_order = 6)]
        verbose: bool,

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

#[derive(clap::Subcommand)]
enum GetMode {
    #[clap(group(
        clap::ArgGroup::new("info")
            .multiple(true)
            .required(true)
            .args(&["name", "class", "brightness", "percent", "max-brightness", "everything", "csv"])
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
        csv: bool,
        /// The backlight or leds device from sysfs to act on. E.g. intel_backlight
        /// If no device name is supplied and unless any other related flag is set
        /// licht will attempt to discover a backlight device in sysfs.
        #[clap(value_parser, long, display_order = 0)]
        device_name: Option<String>,
    },
    /// List availble backlight devices
    List,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.action {
        Action::Get { mode } => match mode {
            GetMode::List => {
                for device in light::discover_all()? {
                    println!("{}", device);
                }
            }
            GetMode::Info {
                name,
                class,
                brightness,
                percent,
                max_brightness,
                everything,
                csv,
                device_name,
            } => {
                let device = if let Some(device_name) = device_name {
                    Light::from_name(&device_name)?
                } else {
                    // "No device name supplied, choosing a backlight device"
                    Light::default()?
                };

                if csv {
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
                    println!("{},", device.get_name());
                } else if class {
                    println!("{},", device.get_class());
                } else if brightness {
                    println!("{},", device.brightness);
                } else if percent {
                    println!("{:.0}%,", device.get_percent() * 100.0f32);
                } else if max_brightness {
                    println!("{}", device.max_brightness);
                }
            }
        },
        Action::Set {
            mode,
            min_brightness,
            all,
            mut verbose,
            dry_run,
            device_name,
        } => {
            if dry_run {
                verbose = true;
            }

            if verbose {
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
            } else if let Some(device_name) = device_name {
                chosen_devices.push(Light::from_name(&device_name)?);
            } else {
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
