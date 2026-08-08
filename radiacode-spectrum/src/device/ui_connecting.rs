use egui::{RichText, Ui};

use radiacode_core::DeviceEndpoint;

pub fn draw_connecting(ui: &mut Ui, endpoint: Option<&DeviceEndpoint>) {
    ui.label(RichText::new("Connecting").size(20.0).strong());
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.spinner();
        let label = endpoint
            .map(|value| value.address_label())
            .unwrap_or("device");
        ui.label(RichText::new(label).monospace().size(16.0));
    });
    ui.add_space(8.0);
    ui.label(RichText::new("Keep the detector powered on.").weak());
}
