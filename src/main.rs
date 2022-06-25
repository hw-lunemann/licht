use clap::Parser;
use simple_logger::SimpleLogger;

mod backlight;
use backlight::Backlight;

mod stepping;
use stepping::Stepping;

#[derive(Parser)]
#[clap(group(clap::ArgGroup::new("stepping-mode").args(&["absolute", "geometric", "parabolic", "blend"]).multiple(false)))]
struct Cli {
    #[clap(value_parser, display_order = 0)]
    /// The backlight class device from sysfs to control. E.g. intel_backlight
    device: String,
    #[clap(value_parser, allow_hyphen_values(true))]
    /// The step used by the chosen stepping. By default it's +-% on the parabolic curve x^2.
    step: i32,
    /// Simply adds the raw step value onto the raw current brightness value
    #[clap(value_parser, name = "absolute", long, display_order = 1)]
    absolute_arg: bool,
    #[clap(skip)]
    absolute: Option<stepping::Absolute>,

    /// Multiplies the current brightness value by <STEP>%
    #[clap(value_parser, name = "geometric", long, display_order = 2)]
    geometric_arg: bool,
    #[clap(skip)]
    geometric: Option<stepping::Geometric>,

    /// Maps the current brightness value onto a the parabolic function
    /// x^exponent and advances it <STEP>% on that function.
    #[clap(value_parser, long, value_name = "EXPONENT", display_order = 3)]
    parabolic: Option<stepping::Parabolic>,

    /// Maps the current birghtness value onto the function
    /// ratio*x^a + (1-m) * (1-(1-x)^(1/b) and advances it <STEP>% on that function.
    /// Recommended parameters for this function are ratio = 0.75, a = 1.8, b = 2.2.
    /// The argument for that would be --blend (0.75,1.8,2.2)
    /// Enter the above function into e.g. desmos or geogebra and
    /// change the parameters to your liking.
    #[clap(value_parser, long, value_name = "(RATIO,A,B)", display_order = 4)]
    blend: Option<stepping::Blend>,

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
}

impl Cli {
    fn get_stepping(&self) -> &dyn Stepping {
        const DEFAULT: stepping::parabolic::Parabolic = stepping::Parabolic { exponent: 2.0f32 };
        self.absolute
            .as_ref()
            .map(|s| s as &dyn Stepping)
            .or_else(|| self.geometric.as_ref().map(|s| s as &dyn Stepping))
            .or_else(|| self.parabolic.as_ref().map(|s| s as &dyn Stepping))
            .or_else(|| self.blend.as_ref().map(|s| s as &dyn Stepping))
            .unwrap_or(&DEFAULT)
    }
}

fn main() -> anyhow::Result<()> {
    let mut cli = Cli::parse();
    if cli.dry_run {
        cli.verbose = true;
    }
    if cli.absolute_arg {
        cli.absolute = Some(stepping::Absolute);
    }
    if cli.geometric_arg {
        cli.geometric = Some(stepping::Geometric);
    }

    if cli.verbose {
        let logger = SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .without_timestamps()
            .init();
        if logger.is_err() {
            eprint!("Error: logger for verbose mode failed to init.");
        }
    }

    let mut backlight = Backlight::new(&cli.device)?;
    log::info!("Device: {}", cli.device);
    log::info!(
        "Current brightness: {} ({:.0}%)",
        backlight.brightness,
        backlight.get_percent() * 100.0f32
    );
    log::info!("Max brightness: {}", backlight.max_brightness);
    backlight.calculate_brightness(cli.step, cli.get_stepping(), cli.min_brightness);

    if !cli.dry_run {
        backlight.write()
    } else {
        Ok(())
    }
}
