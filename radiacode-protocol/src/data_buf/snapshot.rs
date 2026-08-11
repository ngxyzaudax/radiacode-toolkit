use super::decode::DataBufFrame;
use super::group::RecordKind;
use super::records::{DataBufRecord, RareData, RareStatus, RealTimeData, RealTimeRates};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DataBufSnapshot {
    pub rare: Option<RareStatus>,
    pub rates: Option<RealTimeRates>,
}

pub fn latest_snapshot(data: &[u8]) -> DataBufSnapshot {
    snapshot_from_frame(&super::decode::decode_data_buf(data))
}

pub fn snapshot_from_frame(frame: &DataBufFrame) -> DataBufSnapshot {
    let mut snapshot = DataBufSnapshot::default();
    let mut best_rates: Option<(super::seq::Seq, RecordKind, RealTimeRates)> = None;
    for record in &frame.records {
        match record {
            DataBufRecord::Rare(rare) => snapshot.rare = Some(RareStatus::from(*rare)),
            DataBufRecord::RealTime(real_time) => {
                maybe_replace_rates(&mut best_rates, real_time);
            }
            DataBufRecord::Raw(raw) => {
                maybe_replace_legacy_rates(
                    &mut best_rates,
                    raw.header.seq,
                    raw.header.kind,
                    RealTimeRates {
                        count_rate_cps: raw.count_rate_cps.as_f32(),
                        dose_rate_rh: raw.dose_rate_rh.as_f32(),
                        count_rate_err_pct: 0.0,
                        dose_rate_err_pct: 0.0,
                    },
                );
            }
            DataBufRecord::DoseRateDb(db) => {
                maybe_replace_legacy_rates(
                    &mut best_rates,
                    db.header.seq,
                    db.header.kind,
                    RealTimeRates {
                        count_rate_cps: db.count_rate_cps.as_f32(),
                        dose_rate_rh: db.dose_rate_rh.as_f32(),
                        count_rate_err_pct: 0.0,
                        dose_rate_err_pct: db.dose_rate_err_pct,
                    },
                );
            }
            _ => {}
        }
    }
    snapshot.rates = best_rates.map(|(_, _, rates)| rates);
    snapshot
}

pub fn latest_rare_status(data: &[u8]) -> Option<RareStatus> {
    latest_snapshot(data).rare
}

pub fn latest_real_time_rates(data: &[u8]) -> Option<RealTimeRates> {
    latest_snapshot(data).rates
}

pub fn real_time_records(frame: &DataBufFrame) -> impl Iterator<Item = &RealTimeData> {
    frame.records.iter().filter_map(|record| match record {
        DataBufRecord::RealTime(record) => Some(record),
        _ => None,
    })
}

pub fn latest_rare_record(frame: &DataBufFrame) -> Option<&RareData> {
    frame.records.iter().rev().find_map(|record| match record {
        DataBufRecord::Rare(record) => Some(record),
        _ => None,
    })
}

fn maybe_replace_rates(
    best: &mut Option<(super::seq::Seq, RecordKind, RealTimeRates)>,
    record: &RealTimeData,
) {
    maybe_replace_legacy_rates(
        best,
        record.header.seq,
        record.header.kind,
        RealTimeRates::from(*record),
    );
}

fn maybe_replace_legacy_rates(
    best: &mut Option<(super::seq::Seq, RecordKind, RealTimeRates)>,
    seq: super::seq::Seq,
    kind: RecordKind,
    rates: RealTimeRates,
) {
    let replace = best.as_ref().is_none_or(|(best_seq, best_kind, _)| {
        seq.is_newer_than(*best_seq)
            || (seq == *best_seq && kind.monitor_source_rank() > best_kind.monitor_source_rank())
    });
    if replace {
        *best = Some((seq, kind, rates));
    }
}
