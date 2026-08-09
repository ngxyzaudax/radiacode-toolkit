use egui::{Frame, Margin, RichText, Ui};

use crate::theme::{SPACE_MD, SPACE_SM, SPACE_XS};
use crate::view_tab::ViewTab;

pub fn sidebar_content_frame() -> Frame {
    Frame::NONE.inner_margin(Margin::same(SPACE_SM as i8))
}

pub fn draw_sidebar_header(ui: &mut Ui, title: &str) {
    ui.label(RichText::new(title).strong());
    ui.add_space(SPACE_XS);
}

pub fn draw_sidebar_divider(ui: &mut Ui) {
    ui.add_space(SPACE_SM);
    ui.separator();
    ui.add_space(SPACE_SM);
}

pub fn with_page_inset(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Frame::NONE
        .inner_margin(Margin::symmetric(SPACE_MD as i8, SPACE_MD as i8))
        .show(ui, |ui| {
            add_contents(ui);
        });
}

pub fn with_plot_pad(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    ui.add_space(SPACE_SM);
    add_contents(ui);
}

pub fn tab_uses_page_inset(tab: ViewTab) -> bool {
    matches!(
        tab,
        ViewTab::Device | ViewTab::Settings | ViewTab::About | ViewTab::Analysis
    )
}

pub fn tab_uses_plot_pad(tab: ViewTab) -> bool {
    matches!(
        tab,
        ViewTab::Monitor | ViewTab::Spectrum | ViewTab::Spectrogram
    )
}
