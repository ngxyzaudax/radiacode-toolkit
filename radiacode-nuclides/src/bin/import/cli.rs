use std::path::PathBuf;

const MIN_WORKERS: usize = 4;
const MAX_WORKERS: usize = 32;
const FALLBACK_WORKERS: usize = 8;

pub struct ImportOptions {
    pub structure_only: bool,
    pub limit: Option<usize>,
    pub workers: usize,
}

pub fn parse_import_options() -> ImportOptions {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ImportOptions {
        structure_only: args.iter().any(|arg| arg == "--structure-only"),
        limit: flag_value(&args, "--limit="),
        workers: flag_value(&args, "--workers=").unwrap_or_else(default_worker_count),
    }
}

pub(super) fn flag_value(args: &[String], prefix: &str) -> Option<usize> {
    args.iter().find_map(|arg| {
        arg.strip_prefix(prefix)
            .and_then(|value| value.parse::<usize>().ok())
    })
}

fn default_worker_count() -> usize {
    std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(FALLBACK_WORKERS)
        .clamp(MIN_WORKERS, MAX_WORKERS)
}

pub fn data_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/nuclides.json")
}

pub fn decays_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/decays.json")
}
