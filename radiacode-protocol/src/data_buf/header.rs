use std::time::Duration;

use super::group::RecordKind;
use super::seq::Seq;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceTicks(i32);

impl DeviceTicks {
    pub fn new(ticks: i32) -> Self {
        Self(ticks)
    }

    pub fn raw(self) -> i32 {
        self.0
    }

    pub fn as_duration(self) -> Duration {
        let millis = i64::from(self.0).saturating_mul(10).max(0) as u64;
        Duration::from_millis(millis)
    }

    pub fn duration_since(self, origin: Self) -> Duration {
        let millis = i64::from(self.0.saturating_sub(origin.0)).saturating_mul(10);
        Duration::from_millis(millis.max(0) as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::DeviceTicks;

    #[test]
    fn duration_since_negative_offsets() {
        let first = DeviceTicks::new(-3387);
        let second = DeviceTicks::new(-3287);
        assert_eq!(first.duration_since(first).as_millis(), 0);
        assert_eq!(second.duration_since(first).as_secs(), 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecordHeader {
    pub seq: Seq,
    pub kind: RecordKind,
    pub ts: DeviceTicks,
}
