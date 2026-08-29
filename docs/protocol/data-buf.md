# DataBuf records

Each record begins with a 7-byte header:

```text
u8  seq
u8  entity (eid)
u8  group (gid)
i32 ts_offset   (10 ms ticks; wall time = base_time + ts * 10 ms)
... payload ...
```

`base_time` is set at connect to `now + 128 s` after writing `DeviceTime = 0`.

## Record groups

| eid | gid | Group | Payload | Handling |
| --- | --- | --- | --- | --- |
| 0 | 0 | RealTimeData | count_rate f32, dose_rate f32, count_err u16, dose_err u16, flags u16, rt_flags u8 | Decoded |
| 0 | 1 | RawData | count_rate f32, dose_rate f32 | Decoded |
| 0 | 2 | DoseRateDB | count u32, count_rate f32, dose_rate f32, dose_err u16, flags u16 | Decoded |
| 0 | 3 | RareData | duration u32, dose f32, temp u16, charge u16, flags u16 | Decoded |
| 0 | 4 | UserData | 16 bytes | Skipped |
| 0 | 5 | ScheduleData | 16 bytes | Skipped |
| 0 | 6 | AccelData | x u16, y u16, z u16 | Decoded |
| 0 | 7 | Event | event u8, param u8, flags u16 | Decoded |
| 0 | 8 | RawCountRate | count_rate f32, flags u16 | Skipped (6 bytes) |
| 0 | 9 | RawDoseRate | dose_rate f32, flags u16 | Skipped (6 bytes) |
| 1 | 1 | Waveform8 | samples u16, sample_time u32, samples×8 bytes | Skipped |
| 1 | 2 | Waveform16 | samples u16, sample_time u32, samples×16 bytes | Skipped |
| 1 | 3 | Waveform14 | samples u16, sample_time u32, samples×14 bytes | Skipped |

Error percentages: stored as tenths of a percent (`raw / 10.0`).

Temperature: `(raw - 2000) * 0.01` °C. Battery: `charge / 100` %.

## Rate source ranking

`snapshot_from_frame` picks the newest rate record by sequence, breaking ties by group rank:

1. RealTimeData (rank 3)
2. DoseRateDb (rank 2)
3. RawData (rank 1)

`live_rates` and `device_status` consume the snapshot winner. `poll_monitor` plots **RealTimeData only** for the time series; it does not substitute Raw or DB groups into the plot buffer.

## EventId values

| Value | Event |
| --- | --- |
| 0 | PowerOff |
| 1 | PowerOn |
| 2 | LowBatteryShutdown |
| 3 | ChangeDeviceParams |
| 4 | DoseReset |
| 5 | UserEvent |
| 6 | BatteryEmptyAlarm |
| 7 | ChargeStart |
| 8 | ChargeStop |
| 9–14 | Dose rate / dose alarms and offscale |
| 15–16 | Temperature too low / high |
| 17 | TextMessage |
| 18 | MemorySnapshot |
| 19 | SpectrumReset |
| 20–22 | Count rate alarms and offscale |

## StatusFlags bits

| Bit | Meaning |
| --- | --- |
| `0x2000` | Charging |
| `0x4000` | Charge complete |

## Decoder robustness

- Unknown eid/gid: warning with tail hex, up to 4 resync attempts per frame
- Payload underrun on known kind: abort frame decode
- Sanity failure (non-finite rates, out-of-range battery/temp): reject record, continue
- Sequence jump: warning, continue
- Trailing bytes after loop: `TruncatedTail` warning unless last warning was unknown record
