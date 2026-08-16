use std::path::{Path, PathBuf};

use crate::spectrogram::library_meta::load_meta;
use crate::spectrogram::recording;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::storage::list_recordings;

impl SpectrogramState {
    pub fn refresh_history(&mut self) {
        self.history = list_recordings(&self.settings.recordings_dir).unwrap_or_default();
    }

    pub fn filtered_history(&self) -> Vec<crate::spectrogram::model::RecordingEntry> {
        crate::ui::recording::filter_recordings(&self.history, &self.library_filter)
    }

    pub fn can_resume_append(&self) -> bool {
        !self.is_recording() && self.paused_recording_path.is_some()
    }

    pub fn pause_capture(&mut self) -> Result<(), String> {
        let result = recording::pause_capture(self);
        self.sync_from_capture();
        result
    }

    pub fn resume_capture(&mut self) -> Result<(), String> {
        let result = recording::resume_capture(self);
        self.sync_from_capture();
        result
    }

    pub fn start_recording(
        &mut self,
        spectrum: Option<&crate::model::SpectrumView>,
        device_serial: Option<&str>,
    ) -> Result<(), String> {
        let result = recording::start_recording(self, spectrum, device_serial);
        self.sync_from_capture();
        result
    }

    pub fn stop_recording(&mut self) -> Result<(), String> {
        let result = recording::stop_recording(self);
        self.sync_from_capture();
        if result.is_ok()
            && let Some(path) = self.paused_recording_path.clone()
        {
            self.open_library_editor(&path);
        }
        result
    }

    pub fn open_library_editor(&mut self, path: &Path) {
        let fallback = path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("recording");
        let (name, comment) = self
            .history
            .iter()
            .find(|entry| entry.path == path)
            .map(|entry| (entry.name.clone(), entry.comment.clone()))
            .unwrap_or_else(|| {
                let meta = load_meta(path, fallback);
                (meta.name, meta.comment)
            });
        self.library_edit_path = Some(path.to_path_buf());
        self.library_edit_name = name;
        self.library_edit_comment = comment;
    }

    pub fn resume_recording(
        &mut self,
        spectrum: Option<&crate::model::SpectrumView>,
        device_serial: Option<&str>,
    ) -> Result<(), String> {
        let result = recording::resume_recording(self, spectrum, device_serial);
        self.sync_from_capture();
        result
    }

    pub fn request_load(&mut self, path: PathBuf) {
        recording::request_load(self, path);
    }
}
