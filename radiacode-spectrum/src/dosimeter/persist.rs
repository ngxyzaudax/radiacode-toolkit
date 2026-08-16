use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::dosimeter::point::DoseHistoryPoint;
use crate::persist::json_store::{data_dir, load_json, save_json};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StoredDosimeterHistory {
    pub serial: String,
    pub dose_unit_sv: bool,
    pub points: Vec<DoseHistoryPoint>,
}

pub fn load_history(serial: &str) -> Option<StoredDosimeterHistory> {
    let relative_path = history_relative_path(serial);
    let stored: StoredDosimeterHistory = load_json(&relative_path);
    if stored.serial != serial {
        return None;
    }
    let path = data_dir().join(&relative_path);
    debug!(
        serial,
        points = stored.points.len(),
        path = %path.display(),
        "loaded dosimeter history"
    );
    Some(stored)
}

pub fn save_history(stored: &StoredDosimeterHistory) -> std::io::Result<()> {
    save_json(&history_relative_path(&stored.serial), stored)
}

pub fn clear_history(serial: &str) {
    let path = history_path(serial);
    if path.exists()
        && let Err(error) = fs::remove_file(&path)
    {
        warn!(%error, path = %path.display(), "failed to clear dosimeter history");
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

fn history_relative_path(serial: &str) -> String {
    format!("dosimeter/{}.json", sanitize_serial(serial))
}

fn history_path(serial: &str) -> PathBuf {
    data_dir().join(history_relative_path(serial))
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
