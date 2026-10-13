pub const MIN_ENERGY_KEV: f64 = 10.0;
pub const MAX_ENERGY_KEV: f64 = 3000.0;
const DEFAULT_ENERGY_KEV: f64 = 662.0;

pub struct PeakIdState {
    pub energy_kev: f64,
    pub include_all_nuclides: bool,
}

impl PeakIdState {
    pub fn new() -> Self {
        Self {
            energy_kev: DEFAULT_ENERGY_KEV,
            include_all_nuclides: false,
        }
    }

    pub fn set_energy(&mut self, energy_kev: f64) {
        self.energy_kev = clamp_energy(energy_kev);
    }
}

impl Default for PeakIdState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn clamp_energy(energy_kev: f64) -> f64 {
    if energy_kev.is_finite() {
        energy_kev.clamp(MIN_ENERGY_KEV, MAX_ENERGY_KEV)
    } else {
        DEFAULT_ENERGY_KEV
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_ENERGY_KEV, MIN_ENERGY_KEV, PeakIdState, clamp_energy};

    #[test]
    fn set_energy_clamps_to_detector_range() {
        let mut state = PeakIdState::new();
        state.set_energy(5000.0);
        assert_eq!(state.energy_kev, MAX_ENERGY_KEV);
        state.set_energy(1.0);
        assert_eq!(state.energy_kev, MIN_ENERGY_KEV);
        state.set_energy(609.3);
        assert_eq!(state.energy_kev, 609.3);
    }

    #[test]
    fn non_finite_energy_falls_back_to_default() {
        assert_eq!(clamp_energy(f64::NAN), 662.0);
        assert!(!PeakIdState::default().include_all_nuclides);
    }
}
