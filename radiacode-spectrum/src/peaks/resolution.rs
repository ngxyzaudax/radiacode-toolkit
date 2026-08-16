const REFERENCE_ENERGY_KEV: f64 = 662.0;

pub fn fwhm_kev(energy_kev: f64, fwhm_pct_at_662: f64) -> f64 {
    if energy_kev <= 0.0 || fwhm_pct_at_662 <= 0.0 {
        return 1.0;
    }
    let reference_fwhm = REFERENCE_ENERGY_KEV * fwhm_pct_at_662 / 100.0;
    reference_fwhm * (energy_kev / REFERENCE_ENERGY_KEV).sqrt()
}

pub fn fwhm_channels(energy_kev: f64, step_kev: f64, fwhm_pct_at_662: f64) -> usize {
    if step_kev <= 0.0 {
        return 1;
    }
    fwhm_kev(energy_kev, fwhm_pct_at_662)
        .div_euclid(step_kev)
        .max(1.0) as usize
}
