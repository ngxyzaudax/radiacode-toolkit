use egui::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorLeaveChoice {
    Save,
    Discard,
    Stay,
}

pub fn draw_monitor_leave_confirm(ctx: &Context, open: bool) -> Option<MonitorLeaveChoice> {
    if !open {
        return None;
    }
    let mut choice = None;
    egui::Window::new("Unsaved alarm settings")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label("Save alarm and signal settings to the device before leaving Monitor?");
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Save to device").clicked() {
                    choice = Some(MonitorLeaveChoice::Save);
                }
                if ui.button("Discard").clicked() {
                    choice = Some(MonitorLeaveChoice::Discard);
                }
                if ui.button("Stay").clicked() {
                    choice = Some(MonitorLeaveChoice::Stay);
                }
            });
        });
    choice
}
