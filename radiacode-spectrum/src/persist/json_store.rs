use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub fn data_dir() -> PathBuf {
    ProjectDirs::from("com", "radiacode", "radiacode-spectrum")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn load_json<T: DeserializeOwned + Default>(relative_path: &str) -> T {
    let path = data_dir().join(relative_path);
    let Ok(bytes) = fs::read(path) else {
        return T::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

pub fn save_json<T: Serialize>(relative_path: &str, value: &T) -> std::io::Result<()> {
    let path = data_dir().join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
}
