use crate::peaks::model::DetectionParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeakMemoKey {
    pub data_token: u64,
    pub params_hash: u64,
}

impl PeakMemoKey {
    pub fn new(data_token: u64, params: DetectionParams) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        params.sigma_min.to_bits().hash(&mut hasher);
        params.detector_fwhm_pct.to_bits().hash(&mut hasher);
        params.min_net_fraction.to_bits().hash(&mut hasher);
        Self {
            data_token,
            params_hash: hasher.finish(),
        }
    }
}

pub struct PeakMemo {
    key: Option<PeakMemoKey>,
    peaks: Vec<crate::peaks::DetectedPeak>,
}

impl PeakMemo {
    pub fn new() -> Self {
        Self {
            key: None,
            peaks: Vec::new(),
        }
    }

    pub fn get_or_compute<F>(
        &mut self,
        key: PeakMemoKey,
        compute: F,
    ) -> &[crate::peaks::DetectedPeak]
    where
        F: FnOnce() -> Vec<crate::peaks::DetectedPeak>,
    {
        if self.key != Some(key) {
            self.peaks = compute();
            self.key = Some(key);
        }
        &self.peaks
    }
}

impl Default for PeakMemo {
    fn default() -> Self {
        Self::new()
    }
}
