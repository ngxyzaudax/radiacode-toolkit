# RadiaCode Wire Protocol

Reference for the `radiacode-protocol` crate. Field layouts mirror the device configuration INI (`VirtString::Configuration`) and the SFR register directory (`VirtString::SfrFile`).

## Frame layout

Every request and response is length-prefixed:

```text
u32 payload_len LE
u16 command LE
u8  reserved (0)
u8  sequence (0x80..0x9F, wraps mod 32)
... command args ...
```

Responses echo the 4-byte request header, then carry a retcode and payload. Virtual string reads return:

```text
u32 retcode   (1 = OK)
u32 flen      (payload byte count)
... payload ...
```

## Virtual strings

| ID | Name | Purpose |
|----|------|---------|
| 2 | Configuration | Device INI describing message groups and channels |
| 8 | SerialNumber | User-visible serial text |
| 0x100 | DataBuf | Ring buffer of monitor records |
| 0x101 | SfrFile | Self-describing VSFR directory |
| 0x200 | Spectrum | Live histogram |
| 0x202 | EnergyCalib | `[a0, a1, a2]` calibration |
| 0x205 | SpecAccum | Accumulated histogram |

## DataBuf records

Each record begins with a 7-byte header:

```text
u8  seq
u8  entity (eid)
u8  group (gid)
i32 ts_offset   (device time = base + ts_offset * 10 ms)
... payload ...
```

| eid | gid | Group | Payload |
|-----|-----|-------|---------|
| 0 | 0 | RealTimeData | count_rate f32, dose_rate f32, count_err u16, dose_err u16, flags u16, rt_flags u8 |
| 0 | 1 | RawData | count_rate f32, dose_rate f32 |
| 0 | 2 | DoseRateDB | count u32, count_rate f32, dose_rate f32, dose_err u16, flags u16 |
| 0 | 3 | RareData | duration u32, dose f32, temp u16, charge u16, flags u16 |
| 0 | 7 | Event | event u8, param u8, flags u16 |

Monitor plotting uses **RealTimeData only** (gid 0). Raw and DB groups are decoded but never substituted.

Error percentages are stored as tenths of a percent (`raw / 10.0`).

Temperature scaling: `(raw - 2000) * 0.01` °C. Battery: `charge / 100` %.

## Spectrum compression

Header: `duration_sec u32`, `a0/a1/a2 f32`.

- Format 0: raw `u32` counts until buffer end
- Format 1: packed runs — upper 12 bits repeat count, lower 4 bits vlen code (0..=5)

## VSFR units

| Register | Wire unit |
|----------|-----------|
| DR_LEV1/2 | µR/h (÷100 when dose unit is Sv) |
| CR_LEV1/2 | cp10s (×10 for cps, ÷6 for cpm) |
| DS_LEV1/2 | µR accum |
| Real-time dose in DataBuf | R/h |
| Real-time count in DataBuf | cps |

Display conversions live in `radiacode_protocol::rate_units`.

## Catalog validation

On connect, `validate_catalog` compares the static `VirtSfr` table against the device `SFR_FILE`. Drift is logged at warn level; the static catalog remains authoritative for decoding.
