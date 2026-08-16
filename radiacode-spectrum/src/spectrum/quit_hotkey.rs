use egui::{Context, Key};

pub fn quit_hotkey_pressed(ctx: &Context) -> bool {
    if ctx.text_edit_focused() {
        return false;
    }
    ctx.input(|input| input.key_pressed(Key::Q))
}
