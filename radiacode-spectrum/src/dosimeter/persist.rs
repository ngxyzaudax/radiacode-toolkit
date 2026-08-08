use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::dosimeter::point::DoseHistoryPoint;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredDosimeterHistory {
    pub serial: String,
    pub dose_unit_sv: bool,
    pub points: Vec<DoseHistoryPoint>,
}

pub fn load_history(serial: &str) -> Option<StoredDosimeterHistory> {
    let path = history_path(serial);
    let bytes = fs::read(&path).ok()?;
    let stored: StoredDosimeterHistory = serde_json::from_slice(&bytes).ok()?;
    if stored.serial != serial {
        return None;
    }
    debug!(
        serial,
        points = stored.points.len(),
        path = %path.display(),
        "loaded dosimeter history"
    );
    Some(stored)
}

pub fn save_history(stored: &StoredDosimeterHistory) -> std::io::Result<()> {
    let path = history_path(&stored.serial);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = serde_json::to_vec(&stored)?;
    fs::write(path, bytes)
}

pub fn clear_history(serial: &str) {
    let path = history_path(serial);
    if path.exists() {
        if let Err(error) = fs::remove_file(&path) {
            warn!(%error, path = %path.display(), "failed to clear dosimeter history");
        }
    }
}

pub fn history_from_points(
    serial: &str,
    dose_unit_sv: bool,
    points: &VecDeque<DoseHistoryPoint>,
) -> StoredDosimeterHistory {
    StoredDosimeterHistory {
        serial: serial.to_string(),
        dose_unit_sv,
        points: points.iter().copied().collect(),
    }
}

fn history_path(serial: &str) -> PathBuf {
    let safe = sanitize_serial(serial);
    dosimeter_dir().join(format!("{safe}.json"))
}

fn dosimeter_dir() -> PathBuf {
    ProjectDirs::from("com", "radiacode", "radiacode-spectrum")
        .map(|dirs| dirs.data_dir().join("dosimeter"))
        .unwrap_or_else(|| PathBuf::from("dosimeter"))
}

fn sanitize_serial(serial: &str) -> String {
    let safe: String = serial
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if safe.is_empty() {
        "unknown".into()
    } else {
        safe
    }
}
