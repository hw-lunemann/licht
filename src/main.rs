use std::ffi::OsStr;

use anyhow::Context;
use clap::Parser;
use simple_logger::SimpleLogger;

#[macro_use] mod verbose;

mod backlight;
use backlight::Backlight;

mod stepping;
use stepping::Stepping;



#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    action: Action,

    /// The backlight class device from sysfs to act on. E.g. intel_backlight
    /// If no device name is supplied and unless any other related flag is set
    /// licht will attempt to discover a backlight device in sysfs.
    #[clap(value_parser, long, display_order = 0, global = true)]
    device_name: Option<String>,
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

        /// Clamps the brightness to a minimum value.
        #[clap(value_parser, long, default_value("0"), display_order = 5)]
        min_brightness: usize,

        /// Operate on all backlight devices
        #[clap(long)]
        all: bool,

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
        #[clap(long)]
        machine_readable: bool,
    },
    /// List availble backlight devices
    List,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let backlights = Backlight::discover()?;

    let mut backlight = if let Some(device_name) = &cli.device_name {
        backlights
            .iter()
            .find(|backlight| backlight.device_path.file_name().unwrap() == OsStr::new(device_name))
            .context(format!(
                "Could not find device with name \'{}\'",
                device_name
            ))?
    } else {
        // "No device name supplied, choosing a device"
        backlights.first().context("No backlight devices found.")?
    }
    .clone();

    match cli.action {
        Action::Get { mode } => match mode {
            GetMode::List => {
                for device in backlights {
                    println!("{}", device);
                }
            }
            GetMode::Info {
                name,
                class,
                brightness,
                percent,
                max_brightness,
                machine_readable,
            } => {
                if machine_readable {
                    if !name && !brightness && !max_brightness {
                        print!(
                            "{},{},{},{:.0}%,{}",
                            backlight.get_name(),
                            backlight.get_class(),
                            backlight.brightness,
                            backlight.get_percent() * 100.0f32,
                            backlight.max_brightness
                        );
                    } else {
                        if name {
                            print!("{},", backlight.get_name());
                        }
                        if class {
                            print!("{},", backlight.get_class());
                        }
                        if brightness {
                            print!("{},", backlight.brightness);
                        }
                        if percent {
                            print!("{:.0}%,", backlight.get_percent() * 100.0f32);
                        }
                        if max_brightness {
                            print!("{}", backlight.max_brightness);
                        }
                    }
                }
                if !name && !brightness && !max_brightness {
                    println!("{}", backlight);
                } else {
                    if name {
                        println!("{}", backlight.get_name());
                    }
                    if brightness {
                        println!("{}", backlight.brightness);
                    }
                    if percent {
                        println!("{:.0}%", backlight.get_percent() * 100.0f32);
                    }
                    if max_brightness {
                        println!("{}", backlight.max_brightness);
                    }
                }
            }
        },
        Action::Set {
            mode,
            min_brightness,
            all,
            mut verbose,
            dry_run,
        } => {
            if dry_run {
                verbose = true;
            }

            if verbose {
                let logger = SimpleLogger::new()
                    .with_level(log::LevelFilter::Info)
                    .without_timestamps()
                    .init();
                if logger.is_err() {
                    eprint!("Error: logger failed to init.");
                }
            }


            if all {
                for mut backlight in backlights {
                    log::info!("{}", backlight);
                    backlight.calculate_brightness(mode.get_stepping(), min_brightness);
                }
            } else {
                log::info!("{}", backlight);
                backlight.calculate_brightness(mode.get_stepping(), min_brightness);
            }

            if !dry_run {
                backlight.write()?;
            }
        }
    }
    Ok(())
}
