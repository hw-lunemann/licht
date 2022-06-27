use anyhow::Context;
use clap::Parser;
use simple_logger::SimpleLogger;

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
    #[clap(value_parser, long, display_order = 0, global(true))]
    device_name: Option<String>,
}

#[derive(clap::Subcommand)]
enum Action {
    Get {
        #[clap(subcommand)]
        mode: GetMode

    }, 
    Set {
        #[clap(subcommand)]
        mode: SetMode,

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
        absolute: stepping::Absolute
    },
    /// Adds <STEP>% to the current brightness value
    Linear {
        #[clap(flatten)]
        linear: stepping::Linear
    },
    /// Maps the current birghtness value onto the function
    /// ratio*x^a + (1-m) * (1-(1-x)^(1/b) and advances it <STEP>% on that function.
    /// Recommended parameters for this function are ratio = 0.75, a = 1.8, b = 2.2.
    /// The argument for that would be --blend (0.75,1.8,2.2)
    /// Enter the above function into e.g. desmos or geogebra and
    /// change the parameters to your liking.
    Blend {
        #[clap(flatten)]
        blend: stepping::Blend
    },
    /// Multiplies the current brightness value by <STEP>%
    Geometric {
        #[clap(flatten)]
        geometric: stepping::Geometric
    },
    /// Maps the current brightness value onto a the parabolic function
    /// x^exponent and advances it <STEP>% on that function.
    Parabolic {
        #[clap(flatten)]
        parabolic: stepping::Parabolic
    }
}

impl SetMode {
    fn get_stepping(&self) -> &dyn Stepping {
        match &self {
            Self::Absolute { absolute } => absolute,
            Self::Linear { linear } => linear,
            Self::Blend { blend } => blend,
            Self::Geometric { geometric } => geometric,
            Self::Parabolic { parabolic } => parabolic
        }
    }
}

#[derive(clap::Subcommand)]
enum GetMode {
    /// List availble backlight devices
    List,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.action {
        Action::Get { mode } => {
            match mode {
                GetMode::List => {
                    for device_path in Backlight::discover() {
                        println!("{}", Backlight::from_path(&device_path)?);
                    }
                },
            }
        },
        Action::Set { mode, min_brightness, mut verbose, dry_run } => {
            if dry_run {
                verbose = true;
            }

            if verbose {
                let logger = SimpleLogger::new()
                    .with_level(log::LevelFilter::Info)
                    .without_timestamps()
                    .init();
                if logger.is_err() {
                    eprint!("Error: logger for verbose mode failed to init.");
                }
            }

            let mut backlight = if let Some(device_name) = &cli.device_name {
                Backlight::from_name(device_name)
            } else {
                log::info!("No device name supplied, attempting to discover backlight devices.");
                let devices = Backlight::discover();
                if let Some(device_path) = devices.first() {
                    log::info!("Success! Using first device found.");
                    Backlight::from_path(device_path)
                } else {
                    anyhow::bail!("No backlight device supplied or found")
                }
            }?;

            log::info!("{}", backlight);
            backlight.calculate_brightness(
                mode.get_stepping(),
                min_brightness);

            if !dry_run {
                backlight.write()?;
            }
        },
    }
    Ok(())
}
