use egui::Ui;

pub fn scroll_recording_list(ui: &mut Ui, max_height: f32, draw_rows: impl FnOnce(&mut Ui)) {
    let outer_width = ui.available_width();
    egui::ScrollArea::vertical()
        .id_salt("recording_library_list")
        .max_height(max_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.set_max_width(outer_width);
            draw_rows(ui);
        });
}

pub fn draw_recording_row(ui: &mut Ui, draw_row: impl FnOnce(&mut Ui)) {
    let row_width = ui.available_width();
    ui.set_max_width(row_width);
    draw_row(ui);
}
