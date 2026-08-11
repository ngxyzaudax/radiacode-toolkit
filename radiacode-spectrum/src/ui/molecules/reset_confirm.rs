use egui::{Context, Ui};

use crate::ui::atoms::trash_icon::draw_trash_icon_button;
use crate::ui::molecules::confirm_dialog::{
    ConfirmChoice, ConfirmDialogCopy, draw_confirm_dialog, set_dialog_open,
};

pub fn draw_reset_confirm(
    ui: &mut Ui,
    ctx: &Context,
    scope: &'static str,
    enabled: bool,
    tooltip: &'static str,
    copy: ConfirmDialogCopy<'_>,
) -> bool {
    let dialog_id = ui.id().with(scope);
    let clicked = draw_trash_icon_button(ui, enabled)
        .on_hover_text(tooltip)
        .clicked();
    if clicked && enabled {
        set_dialog_open(ctx, dialog_id, true);
    }
    matches!(
        draw_confirm_dialog(ctx, dialog_id, copy),
        Some(ConfirmChoice::Confirm)
    )
}
