use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[cfg(test)]
thread_local! {
    static TEST_DATA_DIR: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub fn use_test_data_dir(path: PathBuf) {
    TEST_DATA_DIR.with(|dir| *dir.borrow_mut() = Some(path));
}

#[cfg(test)]
fn test_data_dir() -> Option<PathBuf> {
    TEST_DATA_DIR.with(|dir| dir.borrow().clone())
}

#[cfg(not(test))]
fn test_data_dir() -> Option<PathBuf> {
    None
}

pub fn data_dir() -> PathBuf {
    test_data_dir().unwrap_or_else(platform_data_dir)
}

fn platform_data_dir() -> PathBuf {
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

#[cfg(test)]
mod tests {
    use super::{data_dir, load_json, save_json, use_test_data_dir};

    #[test]
    fn json_round_trip_in_isolated_data_dir() {
        let temp = tempfile::tempdir().expect("tempdir");
        use_test_data_dir(temp.path().to_path_buf());
        assert_eq!(data_dir(), temp.path());
        save_json("nested/value.json", &vec![1u32, 2, 3]).expect("save");
        let loaded: Vec<u32> = load_json("nested/value.json");
        assert_eq!(loaded, vec![1, 2, 3]);
        let missing: Vec<u32> = load_json("nested/missing.json");
        assert!(missing.is_empty());
    }

    #[test]
    fn corrupt_json_falls_back_to_default() {
        let temp = tempfile::tempdir().expect("tempdir");
        use_test_data_dir(temp.path().to_path_buf());
        std::fs::write(temp.path().join("bad.json"), b"not json").expect("write");
        let loaded: Vec<u32> = load_json("bad.json");
        assert!(loaded.is_empty());
    }
}
