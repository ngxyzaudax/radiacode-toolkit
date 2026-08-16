use std::sync::OnceLock;
use std::time::{Duration, Instant};

use btleplug::platform::Adapter;
use tokio::sync::Mutex;

use crate::adapter::default_adapter;
use crate::ble_error::BleError;

const SESSION_TTL: Duration = Duration::from_secs(120);

struct ScanSession {
    adapter: Adapter,
    scanned_at: Instant,
}

static SCAN_SESSION: OnceLock<Mutex<Option<ScanSession>>> = OnceLock::new();

fn session_lock() -> &'static Mutex<Option<ScanSession>> {
    SCAN_SESSION.get_or_init(|| Mutex::new(None))
}

pub async fn remember_scan_adapter(adapter: &Adapter) {
    let mut guard = session_lock().lock().await;
    *guard = Some(ScanSession {
        adapter: adapter.clone(),
        scanned_at: Instant::now(),
    });
}

pub async fn adapter_for_connect() -> Result<Adapter, BleError> {
    if let Some(session) = session_lock().lock().await.as_ref()
        && session.scanned_at.elapsed() < SESSION_TTL
    {
        return Ok(session.adapter.clone());
    }
    let adapter = default_adapter().await?;
    remember_scan_adapter(&adapter).await;
    Ok(adapter)
}
