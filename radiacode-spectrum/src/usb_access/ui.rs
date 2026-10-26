use egui::{Context, RichText, Ui};
use radiacode_lib::usb::UsbAccessStatus;

use crate::theme::ACCENT;

use super::prompt::{UsbAccessAction, UsbAccessPrompt};

pub fn draw_usb_access_dialog(
    ctx: &Context,
    prompt: &mut UsbAccessPrompt,
) -> Option<UsbAccessAction> {
    let mut action = None;
    let open = egui::Window::new("USB access required")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Radiacode needs permission to open the USB device.")
                    .strong()
                    .color(ACCENT),
            );
            ui.add_space(8.0);
            ui.label(&prompt.message);
            ui.add_space(12.0);
            action = draw_action_buttons(ui, prompt);
            if prompt.installing {
                draw_installing_hint(ui);
            }
        });
    if open.is_none() {
        action = Some(UsbAccessAction::Dismiss);
    }
    action
}

fn draw_action_buttons(ui: &mut Ui, prompt: &UsbAccessPrompt) -> Option<UsbAccessAction> {
    let need_replug = prompt.status == UsbAccessStatus::RuleInstalledNeedReplug;
    ui.horizontal(|ui| {
        let install = ui
            .add_enabled(
                !prompt.installing && !need_replug,
                egui::Button::new("Grant USB access"),
            )
            .clicked()
            .then_some(UsbAccessAction::Install);
        let rescan = (need_replug
            && ui
                .add_enabled(!prompt.installing, egui::Button::new("Rescan and connect"))
                .clicked())
        .then_some(UsbAccessAction::RescanAndConnect);
        let dismiss = ui
            .button("Cancel")
            .clicked()
            .then_some(UsbAccessAction::Dismiss);
        install.or(rescan).or(dismiss)
    })
    .inner
}

fn draw_installing_hint(ui: &mut Ui) {
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label("System password dialog should appear…");
    });
}
