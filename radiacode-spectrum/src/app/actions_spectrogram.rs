use tracing::info;

use super::SpectrumApp;
use super::actions_settings::interval_changed;
use crate::spectrogram::controls_action::SpectrogramControlsAction;

impl SpectrumApp {
    pub(super) fn handle_spectrogram_action(&mut self, action: SpectrogramControlsAction) {
        match action {
            SpectrogramControlsAction::StartRecording => {
                let outcome = self.with_recording_context(|spectrogram, spectrum, serial| {
                    spectrogram.start_recording(spectrum, serial)
                });
                self.apply_spectrogram_outcome(outcome);
            }
            SpectrogramControlsAction::ResumeRecording => {
                let outcome = self.with_recording_context(|spectrogram, spectrum, serial| {
                    spectrogram.resume_recording(spectrum, serial)
                });
                self.apply_spectrogram_outcome(outcome);
            }
            SpectrogramControlsAction::StopRecording => {
                let outcome = self.spectrogram.stop_recording();
                self.apply_spectrogram_outcome(outcome);
            }
            SpectrogramControlsAction::PauseCapture => {
                let outcome = self.spectrogram.pause_capture();
                self.apply_spectrogram_outcome(outcome);
            }
            SpectrogramControlsAction::ResumeCapture => {
                let outcome = self.spectrogram.resume_capture();
                self.apply_spectrogram_outcome(outcome);
            }
            SpectrogramControlsAction::ResetAccumulation => {
                self.spectrogram.reset_accumulation();
                self.apply_spectrogram_outcome(Ok(()));
            }
            SpectrogramControlsAction::CloseLoaded => self.spectrogram.close_loaded(),
            SpectrogramControlsAction::Load(path) => self.spectrogram.request_load(path),
            SpectrogramControlsAction::SettingsChanged => self.sync_settings_from_spectrogram(),
            SpectrogramControlsAction::LibraryChanged => self.refresh_recording_libraries(),
        }
    }

    fn with_recording_context<F>(&mut self, operation: F) -> Result<(), String>
    where
        F: FnOnce(
            &mut crate::spectrogram::SpectrogramState,
            Option<&crate::model::SpectrumView>,
            Option<&str>,
        ) -> Result<(), String>,
    {
        let serial = self
            .state
            .device_info
            .as_ref()
            .map(|info| info.serial.as_str());
        operation(&mut self.spectrogram, self.state.spectrum.as_ref(), serial)
    }

    fn apply_spectrogram_outcome(&mut self, outcome: Result<(), String>) {
        match outcome {
            Ok(()) => self.spectrogram.error.clear(),
            Err(message) => self.spectrogram.error = message,
        }
    }

    fn sync_settings_from_spectrogram(&mut self) {
        let previous_interval = self.settings.spectrogram.capture_interval_secs;
        self.settings.spectrogram = self.spectrogram.settings.clone();
        self.spectrogram.on_settings_changed();
        if interval_changed(
            previous_interval,
            self.spectrogram.settings.capture_interval_secs,
        ) {
            self.sync_capture_interval();
        }
    }

    pub(super) fn refresh_recording_libraries(&mut self) {
        self.spectrogram.refresh_history();
        self.compare
            .refresh_library(&self.spectrogram.settings.recordings_dir);
    }

    pub(super) fn enter_spectrogram_tab(&mut self) {
        info!("entered spectrogram tab");
        self.spectrogram.on_tab_enter();
    }

    pub(super) fn enter_compare_tab(&mut self) {
        self.compare
            .refresh_library(&self.spectrogram.settings.recordings_dir);
    }
}
