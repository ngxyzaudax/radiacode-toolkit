use egui::Context;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmChoice {
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfirmDialogCopy<'a> {
    pub title: &'a str,
    pub message: &'a str,
    pub confirm_label: &'a str,
    pub cancel_label: &'a str,
}

pub fn draw_confirm_dialog(
    ctx: &Context,
    dialog_id: egui::Id,
    copy: ConfirmDialogCopy<'_>,
) -> Option<ConfirmChoice> {
    if !dialog_open(ctx, dialog_id) {
        return None;
    }
    let choice = show_confirm_window(ctx, copy);
    if choice.is_some() {
        set_dialog_open(ctx, dialog_id, false);
    }
    choice
}

pub fn draw_confirm_dialog_open(
    ctx: &Context,
    open: bool,
    copy: ConfirmDialogCopy<'_>,
) -> Option<ConfirmChoice> {
    if !open {
        return None;
    }
    show_confirm_window(ctx, copy)
}

pub fn dialog_open(ctx: &Context, dialog_id: egui::Id) -> bool {
    ctx.data(|data| data.get_temp::<bool>(dialog_id).unwrap_or(false))
}

pub fn set_dialog_open(ctx: &Context, dialog_id: egui::Id, open: bool) {
    ctx.data_mut(|data| data.insert_temp(dialog_id, open));
}

fn show_confirm_window(ctx: &Context, copy: ConfirmDialogCopy<'_>) -> Option<ConfirmChoice> {
    let mut choice = None;
    egui::Window::new(copy.title)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.label(copy.message);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(copy.confirm_label).clicked() {
                    choice = Some(ConfirmChoice::Confirm);
                }
                if ui.button(copy.cancel_label).clicked() {
                    choice = Some(ConfirmChoice::Cancel);
                }
            });
        });
    choice
}
