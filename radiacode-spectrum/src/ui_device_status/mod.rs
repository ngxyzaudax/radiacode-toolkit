mod icons;

use egui::{Color32, Ui};

use crate::device::LinkQuality;
use crate::layout::draw_toolbar;
use crate::model::DeviceInfo;
use crate::theme::{ACCENT, MUTED};
use radiacode_core::TransportKind;

use icons::{battery_color, paint_battery_icon, paint_thermometer_icon};
pub(crate) use icons::{paint_signal_icon, signal_color};

const PENDING_TEXT: &str = "…";
const NOT_AVAILABLE_TEXT: &str = "N/A";

pub fn draw_status_row(ui: &mut Ui, info: &DeviceInfo) {
    draw_toolbar(ui, |ui| {
        ui.spacing_mut().item_spacing.x = 14.0;
        match info.battery_percent {
            Some(battery) => draw_battery_chip(ui, battery),
            None => draw_battery_pending_chip(ui),
        }
        if let Some(temperature) = info.temperature_c {
            draw_temperature_chip(ui, temperature);
        }
        match info.transport {
            TransportKind::Bluetooth => draw_bluetooth_link_row(ui, info.rssi_dbm),
            TransportKind::Usb => draw_usb_link_row(ui),
        }
    });
}

fn draw_bluetooth_link_row(ui: &mut Ui, rssi_dbm: Option<i16>) {
    match rssi_dbm {
        Some(rssi) => {
            draw_link_quality_chip(ui, rssi);
            draw_signal_strength_chip(ui, rssi);
        }
        None => {
            draw_labelled_muted_chip(ui, "Link", PENDING_TEXT);
            draw_labelled_muted_chip(ui, "Signal", PENDING_TEXT);
        }
    }
}

fn draw_usb_link_row(ui: &mut Ui) {
    draw_labelled_muted_chip(ui, "Link", NOT_AVAILABLE_TEXT);
    draw_labelled_muted_chip(ui, "Signal", NOT_AVAILABLE_TEXT);
}

fn draw_labelled_muted_chip(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).small().color(MUTED));
        ui.label(egui::RichText::new(value).strong().color(MUTED));
    });
}

fn draw_labelled_value_chip(ui: &mut Ui, label: &str, value: String, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).small().color(MUTED));
        ui.label(egui::RichText::new(value).strong().color(color));
    });
}

fn draw_battery_pending_chip(ui: &mut Ui) {
    ui.horizontal(|ui| {
        paint_battery_icon(ui, 0.0);
        ui.label(egui::RichText::new(PENDING_TEXT).strong().color(MUTED));
    });
}

fn draw_battery_chip(ui: &mut Ui, percent: f32) {
    let percent = percent.clamp(0.0, 100.0);
    ui.horizontal(|ui| {
        paint_battery_icon(ui, percent);
        ui.label(
            egui::RichText::new(format!("{percent:.1}%"))
                .strong()
                .color(battery_color(percent)),
        );
    });
}

fn draw_temperature_chip(ui: &mut Ui, celsius: f32) {
    ui.horizontal(|ui| {
        paint_thermometer_icon(ui);
        ui.label(
            egui::RichText::new(format!("{celsius:.1} °C"))
                .strong()
                .color(ACCENT),
        );
    });
}

fn draw_link_quality_chip(ui: &mut Ui, rssi_dbm: i16) {
    let quality = LinkQuality::from_rssi(rssi_dbm);
    ui.horizontal(|ui| {
        paint_signal_icon(ui, quality);
        ui.label(
            egui::RichText::new(quality.label())
                .strong()
                .color(quality.color()),
        );
    });
}

fn draw_signal_strength_chip(ui: &mut Ui, rssi_dbm: i16) {
    draw_labelled_value_chip(
        ui,
        "Signal",
        format!("{rssi_dbm} dBm"),
        signal_color(rssi_dbm),
    );
}

#[cfg(test)]
mod tests {
    use super::icons::{battery_color, battery_fill_width};
    use egui::Color32;

    #[test]
    fn battery_color_bands_follow_charge_thresholds() {
        assert_eq!(battery_color(10.0), Color32::from_rgb(220, 90, 90));
        assert_eq!(battery_color(15.0), Color32::from_rgb(220, 90, 90));
        assert_eq!(battery_color(30.0), Color32::from_rgb(230, 170, 70));
        assert_eq!(battery_color(80.0), Color32::from_rgb(110, 190, 120));
    }

    #[test]
    fn battery_fill_width_never_negative() {
        assert_eq!(battery_fill_width(2.0, 50.0), 0.0);
        assert!((battery_fill_width(23.0, 50.0) - 10.0).abs() < f32::EPSILON);
    }
}
