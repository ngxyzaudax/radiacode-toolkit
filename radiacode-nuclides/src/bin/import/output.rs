use std::fs;
use std::path::Path;

use radiacode_nuclides::{Catalog, DecayCatalog};

pub type ImportResult<T> = Result<T, Box<dyn std::error::Error>>;

pub fn existing_nuclide_count(path: &Path) -> usize {
    if !path.exists() {
        return 0;
    }
    let content = fs::read_to_string(path).unwrap_or_default();
    content.matches("\"display_name\"").count()
}

pub fn backup_file(path: &Path) -> ImportResult<()> {
    if !path.exists() {
        return Ok(());
    }
    fs::copy(path, path.with_extension("json.bak"))?;
    Ok(())
}

pub fn write_catalog(path: &Path, catalog: &Catalog) -> ImportResult<()> {
    fs::write(path, serde_json::to_vec_pretty(catalog)?)?;
    Ok(())
}

pub fn write_decay_catalog(path: &Path, catalog: &DecayCatalog) -> ImportResult<()> {
    fs::write(path, serde_json::to_vec_pretty(catalog)?)?;
    Ok(())
}
