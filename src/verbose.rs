pub static mut _LOGGER: Logger = Logger { enabled: false };

pub struct Logger {
    enabled: bool,
}

impl Logger {
    pub fn enable(&'static mut self) {
        self.enabled = true;
    }

    pub const fn is_enabled(&'static self) -> bool {
        self.enabled
    }
}

#[macro_export]
macro_rules! verbose {
    ($($arg:tt)*) => {
        if unsafe { $crate::verbose::_LOGGER.is_enabled() } {
            println!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! verbose_enable {
    () => {
        unsafe { $crate::verbose::_LOGGER.enable() }
    };
}
