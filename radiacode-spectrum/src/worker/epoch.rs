use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
pub struct SessionEpoch {
    live: Arc<AtomicU64>,
    started: u64,
}

impl SessionEpoch {
    pub fn capture(live: &Arc<AtomicU64>) -> Self {
        Self {
            live: Arc::clone(live),
            started: live.load(Ordering::SeqCst),
        }
    }

    pub fn active(&self) -> bool {
        self.live.load(Ordering::SeqCst) == self.started
    }
}

pub fn end_session(live: &Arc<AtomicU64>) -> u64 {
    live.fetch_add(1, Ordering::SeqCst) + 1
}

#[cfg(test)]
mod tests {
    use super::{SessionEpoch, end_session};
    use std::sync::Arc;
    use std::sync::atomic::AtomicU64;

    #[test]
    fn epoch_stays_active_until_session_ends() {
        let live = Arc::new(AtomicU64::new(0));
        let epoch = SessionEpoch::capture(&live);
        assert!(epoch.active());
        assert_eq!(end_session(&live), 1);
        assert!(!epoch.active());
        assert!(SessionEpoch::capture(&live).active());
    }
}
