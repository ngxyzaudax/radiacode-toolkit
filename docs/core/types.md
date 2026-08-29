# Types and errors

## Domain structs

### DeviceMetadata

| Field | Type |
| --- | --- |
| `serial` | User-visible serial |
| `model` | Derived from serial prefix |
| `versions` | `DeviceVersions` (boot + target) |
| `energy_calib` | `[a0, a1, a2]` keV calibration |

`firmware_label()` formats target version for display.

### DeviceStatus

| Field | Type |
| --- | --- |
| `battery_percent` | `Option<f32>` |
| `temperature_c` | `Option<f32>` |
| `rssi_dbm` | `Option<i16>` |

### AlarmLimits / AlarmLimitsUpdate

Thresholds with active display units. Update struct uses `Option` per field for partial writes.

### LiveRates

Single snapshot: dose rate, count rate, error percentages, and unit enums.

### TimedRates

Like `LiveRates` plus `device_ts: DeviceTicks` for plotting and wall-clock conversion.

### AccumulatedDose

From `RareData`: accumulated dose, duration seconds, dose unit.

### MonitorPollSample

Batch result from `poll_monitor` — rate series, optional accumulation, decode health metrics, sequence gaps.

### FirmwareSignature

| Field | Type |
| --- | --- |
| `signature` | `u32` |
| `filename` | Firmware file name |
| `id_string` | Identity string |

## Configuration enums

| Enum | Values |
| --- | --- |
| `DeviceLanguage` | `Russian`, `English` |
| `AlarmSignalMode` | `Once`, `Continuous` |
| `DisplayDirection` | `Auto`, `Right`, `Left` |
| `BacklightOffTime` | 5 s, 10 s, 15 s, 30 s, 2 min, 5 min |
| `SignalFlags` | Per-event sound/vibration enables |

## Discovery enums

| Enum | Values |
| --- | --- |
| `TransportKind` | `Bluetooth`, `Usb` |
| `DeviceEndpoint` | Address or serial key |

## Display units

Re-exported from `radiacode-protocol`:

| Enum | Variants |
| --- | --- |
| `DoseDisplayUnit` | `MicroSievertPerHour`, `MicroRoentgenPerHour` |
| `CountDisplayUnit` | `Cps`, `Cpm` |

Label helpers: `dose_unit_label`, `count_unit_label`, `dose_accum_unit_label`.

Encode/decode for alarm registers: `encode_dose_alarm`, `decode_dose_alarm`, `encode_count_alarm`, `decode_count_alarm`, `encode_dose_accum`, `decode_dose_accum`.

Real-time DataBuf conversion: `dose_display_from_rh`, `count_display_from_cps`.

## DataBufCursor / SeqGap

See [monitor.md](./monitor.md).

## SessionRestore

| Field | Type |
| --- | --- |
| `versions` | `DeviceVersions` |
| `spectrum_format_version` | `u32` |

## Error

```rust
pub enum Error {
    Protocol(ProtocolError),
    DeviceNotFound,
    IncompatibleFirmware { major, minor },
    MonitorDataPending,
    UsbPermissionDenied,
}
```

| Variant | When |
| --- | --- |
| `Protocol` | Wire errors, timeouts, unexpected retcodes |
| `DeviceNotFound` | No matching transport target |
| `IncompatibleFirmware` | Target < 4.8 on first connect |
| `MonitorDataPending` | DataBuf has no rate records yet |
| `UsbPermissionDenied` | Missing udev rule |

Helpers:

| Method | Meaning |
| --- | --- |
| `is_transient()` | Safe to retry (timeout, underrun) |
| `is_connection_lost()` | Link dropped |
| `is_usb_permission_denied()` | Linux USB access |
| `timeout()`, `connection_closed()` | Constructors |

Protocol-level errors (`BufferUnderrun`, `UnexpectedReturnCode`, `VsfrBatchEmpty`, …) are wrapped in `Error::Protocol`.

## Result alias

`type Result<T> = std::result::Result<T, Error>`

Used throughout the core and transport crates.
