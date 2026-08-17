pub fn nearest_index_within(
    pointer_x: f32,
    candidate_xs: &[Option<f32>],
    radius_px: f32,
) -> Option<usize> {
    let radius_sq = radius_px * radius_px;
    candidate_xs
        .iter()
        .enumerate()
        .filter_map(|(index, candidate_x)| {
            let x = candidate_x.as_ref().copied()?;
            let dist_sq = (pointer_x - x).powi(2);
            if dist_sq <= radius_sq {
                Some((index, dist_sq))
            } else {
                None
            }
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(index, _)| index)
}
