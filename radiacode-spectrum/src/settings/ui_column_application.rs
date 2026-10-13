use egui::{RichText, Ui};

use crate::settings::action::SettingsAction;
use crate::settings::state::SettingsState;
use crate::settings::ui_app::{
    draw_app_alerts, draw_app_appearance, draw_app_capture, draw_app_catalogue,
    draw_app_connection, draw_app_matching, draw_app_monitor_window, draw_app_polling,
};
use crate::settings::ui_layout::settings_section;
use crate::theme::MUTED;

type AppSectionDrawer = fn(&mut Ui, &mut SettingsState) -> bool;

struct AppSection {
    title: &'static str,
    hint: &'static str,
    draw: AppSectionDrawer,
}

const APP_SECTIONS: [AppSection; 7] = [
    AppSection {
        title: "Polling",
        hint: "How often live data is refreshed.",
        draw: draw_app_polling,
    },
    AppSection {
        title: "Monitor window",
        hint: "Rolling time span for dose and count rate plots.",
        draw: draw_app_monitor_window,
    },
    AppSection {
        title: "Peak detection and identification",
        hint: "Sensitivity, detector resolution, and nuclide matching tolerance.",
        draw: draw_app_matching,
    },
    AppSection {
        title: "Catalogue preview",
        hint: "Synthetic gamma spectrum resolution in the nuclide catalogue.",
        draw: draw_app_catalogue,
    },
    AppSection {
        title: "Appearance",
        hint: "UI scale for dense layouts.",
        draw: draw_app_appearance,
    },
    AppSection {
        title: "Connection",
        hint: "Startup and remembered devices.",
        draw: draw_app_connection,
    },
    AppSection {
        title: "PC alerts",
        hint: "Repeat detector alarms on this computer.",
        draw: draw_app_alerts,
    },
];

pub fn draw_application_column(
    ui: &mut Ui,
    state: &mut SettingsState,
    recording: bool,
    action: &mut Option<SettingsAction>,
) {
    ui.label(
        RichText::new("Stored on this PC only. Changes save immediately.")
            .small()
            .color(MUTED),
    );
    ui.add_space(6.0);
    draw_capture_section(ui, state, recording, action);
    for section in &APP_SECTIONS {
        draw_app_section(ui, state, section, action);
    }
}

fn draw_capture_section(
    ui: &mut Ui,
    state: &mut SettingsState,
    recording: bool,
    action: &mut Option<SettingsAction>,
) {
    settings_section(
        ui,
        "Spectrogram capture",
        "Interval, library folder, and display.",
        |ui| {
            if draw_app_capture(ui, state, recording) {
                *action = Some(SettingsAction::SpectrogramChanged);
            }
        },
    );
}

fn draw_app_section(
    ui: &mut Ui,
    state: &mut SettingsState,
    section: &AppSection,
    action: &mut Option<SettingsAction>,
) {
    settings_section(ui, section.title, section.hint, |ui| {
        if (section.draw)(ui, state) {
            *action = Some(SettingsAction::AppChanged);
        }
    });
}
