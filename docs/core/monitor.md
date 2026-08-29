# Monitor and live data

## DataBuf read path

Monitor data comes from `VirtString::DataBuf`. The core crate never substitutes transport-specific polling — it decodes frames with `decode_data_buf` from `radiacode-protocol`.

## live_rates

```rust
let units = device.alarm_limits().await?;
let rates = device.live_rates(&units).await?;
```

Returns the **best available rate snapshot** from the latest DataBuf frame:

1. `RealTimeData` (preferred)
2. `DoseRateDb` (fallback)
3. `RawData` (fallback)

Rates are converted to the device's configured display units (`AlarmLimits.dose_unit`, `count_unit`). Returns `Error::MonitorDataPending` when no rate record exists in the buffer.

## poll_monitor

```rust
let mut cursor = DataBufCursor::default();
let (sample, status) = device.poll_monitor(&units, &mut cursor, true).await?;
```

Returns:

| Field | Content |
| --- | --- |
| `sample.rates` | **RealTimeData only** — one `TimedRates` per record in the frame |
| `sample.accumulated` | Latest `RareData` dose accumulation |
| `sample.decode_warnings` | Non-seq-jump, non-sanity warnings |
| `sample.rejected_records` | Sanity-rejected records |
| `sample.resync_count` | Decoder resync attempts |
| `sample.seq_gaps` | Sequence gaps detected this poll |
| `status` | Battery, temperature, RSSI |

Plot history uses RealTimeData exclusively. Snapshot-based APIs (`live_rates`, `device_status`) may fall back to DoseRateDb or RawData when RealTimeData is absent.

## TimedRates

Each RealTimeData record becomes a `TimedRates` with:

- `device_ts` — relative `DeviceTicks` from the record header
- Display-unit dose and count rates plus error percentages

Convert to wall time: `device.wall_clock(timed.device_ts)`.

## DeviceStatus

| Field | Source |
| --- | --- |
| `battery_percent` | Latest `RareData` charge field |
| `temperature_c` | `RareData` temp, or `VirtSfr::TempDegC` fallback |
| `rssi_dbm` | Transport link RSSI (Bluetooth only) |

`merge_status(carry, fresh)` keeps the last known value when a field is missing in the fresh read.

## DataBufCursor

Tracks sequence continuity across polls:

```rust
let gaps = cursor.observe_frame(&frame);
```

| `SeqGap` field | Meaning |
| --- | --- |
| `expected`, `got` | Sequence bytes |
| `lost` | Records missed (when not a reset) |
| `reset` | Device buffer wrapped or rewound |

`cursor.reset()` clears state after a deliberate reconnect. `total_gaps` and `total_lost` accumulate non-reset gaps.

Gaps larger than 32 records are treated as buffer resets, not loss.

## Display unit conversion

RealTimeData wire units:

- Count rate: **cps**
- Dose rate: **R/h**

Conversion to configured display units uses `dose_display_from_rh` and `count_display_from_cps` (re-exported from protocol). Alarm limit reads/writes use separate encode/decode paths documented in [protocol/vsfr.md](../protocol/vsfr.md).

## Recommended poll loop

```rust
let units = device.alarm_limits().await?;
let mut cursor = DataBufCursor::default();

loop {
    let (sample, status) = device.poll_monitor(&units, &mut cursor, true).await?;
    for rate in &sample.rates {
        let when = device.wall_clock(rate.device_ts);
        // use rate.dose_rate, rate.count_rate, when
    }
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

Drain or tolerate `MonitorDataPending` on the first poll after connect when the buffer is still empty.
