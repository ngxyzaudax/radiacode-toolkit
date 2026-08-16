use std::collections::HashMap;

use radiacode_nuclides::GammaLine;

use super::parse::{parse_gamma_rows, parse_xray_rows};
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

pub fn fetch_radiations(slug: &str) -> Vec<GammaLine> {
    let gamma_url = format!("{API_BASE}?fields=decay_rads&rad_types=g&nuclides={slug}");
    let xray_url = format!("{API_BASE}?fields=decay_rads&rad_types=x&nuclides={slug}");
    let mut radiations = parse_gamma_rows(&fetch_csv(&gamma_url).unwrap_or_default());
    radiations.extend(parse_xray_rows(&fetch_csv(&xray_url).unwrap_or_default()));
    radiations
}
