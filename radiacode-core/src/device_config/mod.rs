mod io;
mod types;

pub use io::{apply_device_config, load_device_config, sync_device_clock};
pub use types::{AlarmSignalMode, BacklightOffTime, DeviceConfig, DisplayDirection, SignalFlags};
