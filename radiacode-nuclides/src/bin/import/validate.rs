use radiacode_nuclides::{Catalog, DecayCatalog, Nuclide};

pub fn validate_catalog(catalog: &Catalog, min_count: usize) -> Result<(), String> {
    if catalog.nuclides.len() < min_count {
        return Err(format!(
            "nuclide count {} below minimum {}",
            catalog.nuclides.len(),
            min_count
        ));
    }
    assert_half_life("Ac-228", catalog, 22140.0, 1.0)?;
    assert_line_intensity("Lu-176", catalog, 201.83, 78.0, 2.0)?;
    assert_line_intensity("Lu-176", catalog, 306.78, 93.6, 2.0)?;
    assert_dominant_line("Ir-192", catalog, 316.506, 468.068)?;
    assert_line_intensity("Cs-137", catalog, 661.657, 85.1, 2.0)?;
    assert_present(catalog, "Tc-99m")?;
    assert_present(catalog, "Ba-137m")?;
    assert_present(catalog, "Pa-234m")?;
    assert_no_stable_with_gammas(catalog)?;
    Ok(())
}

pub fn validate_topology(catalog: &DecayCatalog) -> Result<(), String> {
    if catalog.entries.is_empty() {
        return Err("decays catalog is empty".into());
    }
    Ok(())
}

fn assert_half_life(
    name: &str,
    catalog: &Catalog,
    expected_secs: f64,
    tolerance_secs: f64,
) -> Result<(), String> {
    let nuclide = find_nuclide(catalog, name)?;
    let actual = nuclide
        .half_life_secs
        .ok_or_else(|| format!("{name} missing half-life"))?;
    if (actual - expected_secs).abs() > tolerance_secs {
        return Err(format!(
            "{name} half-life {actual} s, expected {expected_secs} s"
        ));
    }
    Ok(())
}

fn assert_line_intensity(
    name: &str,
    catalog: &Catalog,
    energy_kev: f64,
    expected_pct: f64,
    tolerance_pct: f64,
) -> Result<(), String> {
    let nuclide = find_nuclide(catalog, name)?;
    let line = nuclide
        .gammas
        .iter()
        .find(|line| (line.energy_kev - energy_kev).abs() < 0.2)
        .ok_or_else(|| format!("{name} missing {energy_kev} keV line"))?;
    if (line.intensity_pct - expected_pct).abs() > tolerance_pct {
        return Err(format!(
            "{name} {energy_kev} keV intensity {}%, expected {}%",
            line.intensity_pct, expected_pct
        ));
    }
    Ok(())
}

fn assert_dominant_line(
    name: &str,
    catalog: &Catalog,
    dominant_kev: f64,
    secondary_kev: f64,
) -> Result<(), String> {
    let nuclide = find_nuclide(catalog, name)?;
    let dominant = max_intensity_near(nuclide, dominant_kev)?;
    let secondary = max_intensity_near(nuclide, secondary_kev)?;
    if dominant <= secondary {
        return Err(format!(
            "{name} dominant line {dominant_kev} keV ({dominant}%) not above {secondary_kev} keV ({secondary}%)"
        ));
    }
    Ok(())
}

fn max_intensity_near(nuclide: &Nuclide, energy_kev: f64) -> Result<f64, String> {
    nuclide
        .gammas
        .iter()
        .filter(|line| (line.energy_kev - energy_kev).abs() < 0.2)
        .map(|line| line.intensity_pct)
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or_else(|| format!("{} missing line near {energy_kev} keV", nuclide.display_name))
}

fn assert_present(catalog: &Catalog, name: &str) -> Result<(), String> {
    find_nuclide(catalog, name).map(|_| ())
}

fn assert_no_stable_with_gammas(catalog: &Catalog) -> Result<(), String> {
    for nuclide in &catalog.nuclides {
        let stable = nuclide
            .half_life_text
            .eq_ignore_ascii_case("stable")
            || nuclide.half_life_text.eq_ignore_ascii_case("STABLE");
        if stable && !nuclide.gammas.is_empty() {
            return Err(format!(
                "{} marked stable but has {} gamma lines",
                nuclide.display_name,
                nuclide.gammas.len()
            ));
        }
    }
    Ok(())
}

fn find_nuclide<'a>(catalog: &'a Catalog, name: &str) -> Result<&'a Nuclide, String> {
    catalog
        .nuclides
        .iter()
        .find(|nuclide| nuclide.display_name == name)
        .ok_or_else(|| format!("missing nuclide {name}"))
}
