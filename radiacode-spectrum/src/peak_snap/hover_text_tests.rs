use crate::peak_snap::hover_text::peak_hover_text;
use crate::peaks::detect_fixtures::fixture_analysis;

#[test]
fn hover_text_lists_label_energy_and_net_area() {
    let analysis = fixture_analysis("thoriated_rods");
    let identification = analysis.identifications.first().expect("identification");
    let text = peak_hover_text(identification);
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines.len() >= 3);
    assert_eq!(
        lines[1],
        format!("{:.1} keV", identification.peak.energy_kev)
    );
    assert!(lines[2].starts_with("Net area: "));
}

#[test]
fn hover_text_appends_candidate_line_when_matched() {
    let analysis = fixture_analysis("thoriated_rods");
    let matched = analysis
        .identifications
        .iter()
        .find(|identification| !identification.candidates.is_empty())
        .expect("matched identification");
    let text = peak_hover_text(matched);
    assert_eq!(text.lines().count(), 4);
    assert!(text.lines().last().unwrap().contains("Δ"));
}
