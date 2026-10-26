# VSFR registers

VSFR (virtual special-function register) addresses mirror the device `SFR_FILE`. The static catalog in `radiacode-lib` is validated against the device directory on connect; drift is logged at warn level.

## Common registers

| Address | Name | Kind |
| --- | --- | --- |
| `0x0500` | DeviceCtrl | u32 |
| `0x0502` | DeviceLang | bool (0=ru, 1=en) |
| `0x0503` | DeviceOn | bool |
| `0x0504` | DeviceTime | u32 seconds |
| `0x0511` | DispBrt | u8 (0..9) |
| `0x0513` | DispOffTime | u32 |
| `0x0515` | DispDir | u8 |
| `0x0520` | SoundCtrl | u32 bitmask |
| `0x0522` | SoundOn | u8 |
| `0x0530` | VibroCtrl | u32 bitmask |
| `0x0531` | VibroOn | u8 |
| `0x0545` | LedsOn | u8 |
| `0x05E0` | AlarmMode | u32 |
| `0x8000` | DR_LEV1 | u32 µR/h |
| `0x8001` | DR_LEV2 | u32 µR/h |
| `0x8004` | DS_UNITS | u8 (0=R, 1=Sv) |
| `0x8008` | CR_LEV1 | u32 cp10s |
| `0x8009` | CR_LEV2 | u32 cp10s |
| `0x8013` | CR_UNITS | u8 (0=cps, 1=cpm) |
| `0x8014` | DS_LEV1 | u32 µR accum |
| `0x8015` | DS_LEV2 | u32 µR accum |
| `0x8024` | TEMP_degC | f32 |

Service and calibration registers (`Ms*`, `VBias`, `CompLev`, `CalibMode`, `Dpot*`, system IDs) are defined as addresses only; no high-level write helpers are exposed.

## Batch read wire format

Request:

```text
u32 count
count × u32 register_id
```

Response:

```text
u32 valid_flags     (bit i set → register i returned)
... sparse or dense u32 words ...
```

When `valid_flags != (1 << count) - 1`, missing slots are filled by sequential `RdVirtSfr` fallback.

## Batch write wire format

```text
u32 count
count × u32 register_id
count × u32 value
```

Response: `valid_flags u32` must equal `(1 << count) - 1`.

## Sub-word extraction

Batch and single-read u32 words may carry sub-word values in either the low or high bytes. Extraction picks the non-zero side when only one is populated:

| Kind | Rule |
| --- | --- |
| u8 / bool | low byte unless high byte alone is non-zero |
| u16 / i16 | low halfword unless high halfword alone is non-zero |
| u32 / i32 / f32 | full word |

Kind resolution prefers the device `SFR_FILE` entry (`Size=` / `Type=`), then the static catalog.

## Display unit scaling

| Register | Wire unit | Display |
| --- | --- | --- |
| DR_LEV1/2 | µR/h | ÷100 when dose unit is Sv |
| CR_LEV1/2 | cp10s | ×10 for cps, ÷6 for cpm |
| DS_LEV1/2 | µR accum | ÷100 when dose unit is Sv |
| Real-time dose in DataBuf | R/h | converted in `rate_units` |
| Real-time count in DataBuf | cps | converted in `rate_units` |

Decode count alarm: `raw / 10`, then ×60 for cpm. Encode is the inverse.

Accumulated dose alarms are stored and reported in micro-units (µSv or µR depending on `DS_UNITS`).
