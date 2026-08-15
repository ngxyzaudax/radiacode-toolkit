use egui::Ui;

pub fn draw_query_search(ui: &mut Ui, query: &mut String, hint: &str) -> bool {
    ui.add(egui::TextEdit::singleline(query).hint_text(hint))
        .changed()
}
