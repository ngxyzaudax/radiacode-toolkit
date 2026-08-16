#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MonitorLinkHealth {
    pub decode_warnings: u64,
    pub rejected_records: u64,
    pub resync_count: u64,
    pub seq_gaps: u64,
    pub lost_records: u64,
}

impl MonitorLinkHealth {
    pub fn has_issues(self) -> bool {
        self.decode_warnings > 0
            || self.rejected_records > 0
            || self.resync_count > 0
            || self.seq_gaps > 0
            || self.lost_records > 0
    }

    pub fn summary(self) -> String {
        if !self.has_issues() {
            return "Link health: clean".into();
        }
        format!(
            "Link health: {} decode warnings, {} rejected, {} resyncs, {} seq gaps, {} lost records",
            self.decode_warnings,
            self.rejected_records,
            self.resync_count,
            self.seq_gaps,
            self.lost_records
        )
    }
}
