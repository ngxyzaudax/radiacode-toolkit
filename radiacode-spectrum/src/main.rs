mod about;
mod analysis;
mod app;
mod app_config;
mod catalogue;
mod device;
mod dosimeter;
mod energy;
mod events;
mod icon;
mod identify;
mod layout;
mod logging;
mod model;
mod monitor;
mod monitor_window;
mod pc_alarm;
mod peak_overlay;
mod peaks;
mod persist;
mod plot_hover;
mod plot_style;
mod scale;
mod settings;
mod smooth;
mod spectrogram;
mod spectrum;
mod synthetic_spectrum;
mod tabs;
mod theme;
mod ui;
mod ui_chrome;
mod ui_device_status;
mod ui_disconnected;
mod ui_plot;
mod ui_toolbar;
mod usb_access;
mod view_tab;
mod window;
mod worker;
mod worker_ops;

use std::process::ExitCode;

use app::SpectrumApp;
use icon::{APP_ID, app_icon};
use tracing::{error, info};
use window::{min_inner_size, startup_inner_size, startup_viewport_builder};

fn main() -> ExitCode {
    logging::init();
    info!("radiacode-spectrum starting");
    let options = eframe::NativeOptions {
        viewport: startup_viewport_builder()
            .with_title("Radiacode")
            .with_app_id(APP_ID)
            .with_icon(app_icon()),
        centered: true,
        persist_window: false,
        window_builder: Some(Box::new(|builder| {
            builder
                .with_inner_size(startup_inner_size())
                .with_min_inner_size(min_inner_size())
        })),
        ..Default::default()
    };
    match eframe::run_native(
        APP_ID,
        options,
        Box::new(|_cc| Ok(Box::new(SpectrumApp::new()))),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!(%error, "failed to start gui");
            ExitCode::FAILURE
        }
    }
}
