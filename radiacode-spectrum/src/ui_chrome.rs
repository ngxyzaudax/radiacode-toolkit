use egui::{Frame, Margin, Ui};

use crate::theme::SPACE_MD;
use crate::view_tab::ViewTab;

pub fn with_page_inset(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Frame::NONE
        .inner_margin(Margin::symmetric(SPACE_MD as i8, SPACE_MD as i8))
        .show(ui, |ui| {
            add_contents(ui);
        });
}

pub fn with_plot_pad(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    Frame::NONE
        .inner_margin(Margin::symmetric(SPACE_MD as i8, 0))
        .show(ui, |ui| {
            add_contents(ui);
        });
}

pub fn tab_uses_page_inset(tab: ViewTab) -> bool {
    matches!(
        tab,
        ViewTab::Device | ViewTab::Settings | ViewTab::About | ViewTab::Analysis | ViewTab::Catalogue
    )
}

pub fn tab_uses_plot_pad(tab: ViewTab) -> bool {
    matches!(tab, ViewTab::Monitor | ViewTab::Spectrum | ViewTab::Spectrogram)
}
