use std::collections::HashMap;

use rayon::prelude::*;

use super::isomer::RadiationBundle;
use super::{API_BASE, USER_AGENT};

pub fn fetch_csv(url: &str) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    let response = ureq::get(url).header("User-Agent", USER_AGENT).call()?;
    let body = response.into_body().read_to_string()?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(body.as_bytes());
    let headers = reader.headers()?.clone();
    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record?;
        let mut map = HashMap::new();
        for (index, field) in record.iter().enumerate() {
            if let Some(name) = headers.get(index) {
                map.insert(name.to_string(), field.to_string());
            }
        }
        rows.push(map);
    }
    Ok(rows)
}

pub fn fetch_radiation_bundle(slug: &str) -> RadiationBundle {
    let [gamma_rows, xray_rows, beta_minus_rows, beta_plus_rows]: [Vec<_>; 4] =
        ["g", "x", "bm", "bp"]
            .par_iter()
            .map(|rad_type| fetch_decay_rows(slug, rad_type))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| unreachable!());
    RadiationBundle {
        gamma_rows,
        xray_rows,
        beta_minus_rows,
        beta_plus_rows,
    }
}

pub fn fetch_radiation_bundles(slugs: &[String]) -> HashMap<String, RadiationBundle> {
    slugs
        .par_iter()
        .map(|slug| (slug.clone(), fetch_radiation_bundle(slug)))
        .collect()
}

fn fetch_decay_rows(slug: &str, rad_type: &str) -> Vec<HashMap<String, String>> {
    let url = format!("{API_BASE}?fields=decay_rads&rad_types={rad_type}&nuclides={slug}");
    fetch_csv(&url).unwrap_or_default()
}
