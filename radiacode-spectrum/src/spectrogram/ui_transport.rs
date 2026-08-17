use egui::{Color32, RichText, Ui, Vec2};

use crate::model::ConnectionState;
use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::transport_gating::pause_enabled;
use crate::spectrogram::ui_transport_reset::draw_transport_reset;

pub(crate) const TRANSPORT_SIZE: Vec2 = Vec2::new(34.0, 28.0);

const RECORD_RED: Color32 = Color32::from_rgb(220, 55, 55);
const RECORD_ACTIVE: Color32 = Color32::from_rgb(255, 80, 80);
const STOP_FILL: Color32 = Color32::from_rgb(52, 56, 64);

pub fn draw_transport(
    ui: &mut Ui,
    state: &SpectrogramState,
    connection: ConnectionState,
) -> Option<SpectrogramControlsAction> {
    let connected = connection == ConnectionState::Connected;
    let recording = state.is_recording();
    let capture_paused = state.is_capture_paused();
    let can_append = state.can_resume_append();
    let ctx = ui.ctx().clone();
    let mut action = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if transport_record(ui, connected, recording) {
            action = Some(SpectrogramControlsAction::StartRecording);
        }
        if transport_pause(ui, connected, recording, capture_paused) {
            action = Some(SpectrogramControlsAction::PauseCapture);
        }
        if transport_resume(ui, connected, capture_paused, can_append, recording) {
            action = Some(if can_append && !recording {
                SpectrogramControlsAction::ResumeRecording
            } else {
                SpectrogramControlsAction::ResumeCapture
            });
        }
        if transport_stop(ui, connected, recording) {
            action = Some(SpectrogramControlsAction::StopRecording);
        }
        ui.add_space(4.0);
        if draw_transport_reset(ui, &ctx, recording) {
            action = Some(SpectrogramControlsAction::ResetAccumulation);
        }
    });
    action
}

fn transport_record(ui: &mut Ui, connected: bool, recording: bool) -> bool {
    let fill = if recording { RECORD_ACTIVE } else { RECORD_RED };
    ui.add_enabled(
        connected && !recording,
        egui::Button::new(RichText::new("●").size(15.0))
            .fill(fill)
            .min_size(TRANSPORT_SIZE),
    )
    .on_hover_text("Record new")
    .clicked()
}

fn transport_pause(ui: &mut Ui, connected: bool, recording: bool, capture_paused: bool) -> bool {
    ui.add_enabled(
        pause_enabled(connected, recording, capture_paused),
        egui::Button::new(RichText::new("⏸").size(14.0)).min_size(TRANSPORT_SIZE),
    )
    .on_hover_text("Pause recording")
    .clicked()
}

fn transport_resume(
    ui: &mut Ui,
    connected: bool,
    capture_paused: bool,
    can_append: bool,
    recording: bool,
) -> bool {
    let resume_capture = connected && capture_paused && (recording || !can_append);
    let resume_append = connected && can_append && !recording;
    let enabled = resume_capture || resume_append;
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new("▶").size(14.0)).min_size(TRANSPORT_SIZE),
    )
    .on_hover_text(if resume_append {
        "Resume last recording"
    } else {
        "Resume capture"
    })
    .clicked()
}

fn transport_stop(ui: &mut Ui, connected: bool, recording: bool) -> bool {
    ui.add_enabled(
        connected && recording,
        egui::Button::new(RichText::new("■").size(14.0))
            .fill(STOP_FILL)
            .min_size(TRANSPORT_SIZE),
    )
    .on_hover_text("Stop and save")
    .clicked()
}
