use egui::{ScrollArea, Ui};

pub fn page_scroll(ui: &mut Ui, id: &str, add_contents: impl FnOnce(&mut Ui)) {
    ScrollArea::vertical()
        .id_salt(id)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            add_contents(ui);
        });
}
