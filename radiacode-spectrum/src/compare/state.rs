use std::path::{Path, PathBuf};

use crate::compare::selection::{rebuild_samples, selection_status};
use crate::compare::spectrum::{CollapsedSpectrum, collapse_series};
use crate::compare::subtract::Subtraction;
use crate::peaks::PeakMemo;
use crate::smooth::DEFAULT_SMOOTHING_WINDOW;
use crate::spectrogram::model::RecordingEntry;
use crate::spectrogram::storage::{list_recordings, load_recording};

#[derive(Debug, Clone)]
pub struct ComparedSample {
    pub spectrum: CollapsedSpectrum,
    pub subtraction: Option<Subtraction>,
}

pub struct CompareState {
    pub library: Vec<RecordingEntry>,
    pub library_filter: String,
    pub background_path: Option<PathBuf>,
    pub sample_paths: Vec<PathBuf>,
    pub background: Option<CollapsedSpectrum>,
    pub samples: Vec<ComparedSample>,
    pub smooth_window: usize,
    pub outline_only: bool,
    pub subtract_background: bool,
    pub show_peaks: bool,
    pub status: String,
    pub error: String,
    pub pane_open: bool,
    pub peak_memo: PeakMemo,
}

impl CompareState {
    pub fn new() -> Self {
        Self {
            library: Vec::new(),
            library_filter: String::new(),
            background_path: None,
            sample_paths: Vec::new(),
            background: None,
            samples: Vec::new(),
            smooth_window: DEFAULT_SMOOTHING_WINDOW,
            outline_only: false,
            subtract_background: false,
            show_peaks: false,
            status: String::new(),
            error: String::new(),
            pane_open: true,
            peak_memo: PeakMemo::new(),
        }
    }

    pub fn refresh_library(&mut self, recordings_dir: &str) {
        self.library = list_recordings(recordings_dir).unwrap_or_default();
    }

    pub fn filtered_library(&self) -> Vec<RecordingEntry> {
        crate::ui::recording::filter_recordings(&self.library, &self.library_filter)
    }

    pub fn is_background(&self, path: &Path) -> bool {
        self.background_path
            .as_ref()
            .is_some_and(|item| item == path)
    }

    pub fn sample_index(&self, path: &Path) -> Option<usize> {
        self.sample_paths.iter().position(|item| item == path)
    }

    pub fn set_background(&mut self, entry: &RecordingEntry) {
        if self.sample_index(&entry.path).is_some() {
            return;
        }
        self.background_path = Some(entry.path.clone());
        self.reload_selection();
    }

    pub fn toggle_sample(&mut self, entry: &RecordingEntry) {
        if self.is_background(&entry.path) {
            return;
        }
        if let Some(index) = self.sample_index(&entry.path) {
            self.sample_paths.remove(index);
        } else {
            self.sample_paths.push(entry.path.clone());
        }
        self.reload_selection();
    }

    pub fn remove_sample_at(&mut self, index: usize) {
        if index >= self.sample_paths.len() {
            return;
        }
        self.sample_paths.remove(index);
        self.reload_selection();
    }

    pub fn clear_background(&mut self) {
        self.background_path = None;
        self.reload_selection();
    }

    fn reload_selection(&mut self) {
        self.error.clear();
        self.status.clear();
        self.background = self
            .background_path
            .as_ref()
            .and_then(|path| self.load_collapsed(path));
        let loaded = self
            .sample_paths
            .iter()
            .map(|path| self.load_collapsed(path))
            .collect();
        let (samples, error) = rebuild_samples(loaded, self.background.as_ref());
        self.samples = samples;
        self.error = error;
        self.status = selection_status(self.background.is_some(), self.samples.len());
    }

    fn load_collapsed(&self, path: &PathBuf) -> Option<CollapsedSpectrum> {
        let entry = self.library.iter().find(|item| &item.path == path)?;
        let series = load_recording(path).ok()?;
        Some(collapse_series(&series, entry))
    }
}
