mod about;
mod app;
mod app_config;
mod compare;
mod device;
mod dosimeter;
mod energy;
#[cfg(test)]
mod energy_tests;
mod events;
mod icon;
mod identify;
mod layout;
mod logging;
mod model;
#[cfg(test)]
mod model_tests;
mod monitor;
mod monitor_window;
mod pc_alarm;
mod peak_id;
mod peak_overlay;
mod peak_snap;
mod peaks;
mod persist;
mod plot_hover;
#[cfg(test)]
mod plot_hover_tests;
mod plot_style;
mod plot_zoom;
mod scale;
mod settings;
mod smooth;
mod sources_tab;
mod spectrogram;
mod spectrum;
mod synthetic_spectrum;
mod tabs;
mod theme;
mod time_format;
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

use std::process::ExitCode;

use app::SpectrumApp;
use icon::{APP_ID, app_icon};
use tracing::{error, info};
use window::{min_inner_size, startup_inner_size, startup_viewport_builder};

fn main() -> ExitCode {
    logging::init();
    std::panic::set_hook(Box::new(|panic| {
        error!(%panic, "application panic");
        let backtrace = std::backtrace::Backtrace::force_capture();
        error!(%backtrace, "panic backtrace");
        let _ = std::fs::write(
            "/tmp/radiacode-spectrum-panic.log",
            format!("{panic}\n{backtrace}"),
        );
    }));
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
