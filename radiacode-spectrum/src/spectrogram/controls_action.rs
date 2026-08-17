use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpectrogramControlsAction {
    StartRecording,
    StopRecording,
    PauseCapture,
    ResumeCapture,
    ResumeRecording,
    ResetAccumulation,
    CloseLoaded,
    Load(PathBuf),
    SettingsChanged,
    LibraryChanged,
}
