mod apply;
mod io;
mod signal_flags;
mod types;

pub(crate) use apply::alarm_register_pairs;
pub use apply::{apply_device_config, sync_device_clock};
pub use io::load_device_config;
pub(crate) use io::{CONFIG_REGISTERS, alarm_limits_from_words};
pub use signal_flags::SignalFlags;
pub use types::{AlarmSignalMode, BacklightOffTime, DeviceConfig, DisplayDirection};
