use crate::ui::molecules::confirm_dialog::ConfirmDialogCopy;

pub const DOSE_RESET: ConfirmDialogCopy<'static> = ConfirmDialogCopy {
    title: "Reset accumulated dose",
    message: "Reset accumulated dose on the device?",
    confirm_label: "Reset",
    cancel_label: "Cancel",
};

pub const SPECTRUM_RESET: ConfirmDialogCopy<'static> = ConfirmDialogCopy {
    title: "Reset spectrum accumulation",
    message: "Reset spectrum accumulation on the device?",
    confirm_label: "Reset",
    cancel_label: "Cancel",
};

pub const SPECTROGRAM_RESET: ConfirmDialogCopy<'static> = ConfirmDialogCopy {
    title: "Reset spectrogram accumulation",
    message: "Clear accumulated spectrogram rows and baseline?",
    confirm_label: "Reset",
    cancel_label: "Cancel",
};

pub const SPECTROGRAM_LIBRARY_DELETE: ConfirmDialogCopy<'static> = ConfirmDialogCopy {
    title: "Delete recording",
    message: "Delete this spectrogram recording from the library?",
    confirm_label: "Delete",
    cancel_label: "Cancel",
};

pub const LOAD_SETTINGS: ConfirmDialogCopy<'static> = ConfirmDialogCopy {
    title: "Unsaved changes",
    message: "You have unsaved changes. Load from device and discard your edits?",
    confirm_label: "Load anyway",
    cancel_label: "Keep editing",
};
