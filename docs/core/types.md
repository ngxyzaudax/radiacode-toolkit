# Types and errors

## Domain structs

### DeviceMetadata

| Field | Type |
| --- | --- |
| `serial` | User-visible serial |
| `model` | Derived from serial prefix |
| `versions` | `DeviceVersions` (boot + target) |
| `energy_calibration` | `EnergyCalibration` (`a0`, `a1`, `a2`) in keV |

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

Defined in `radiacode-lib` (`units`, re-exported at the crate root):

| Enum | Variants |
| --- | --- |
| `DoseDisplayUnit` | `MicroSievertPerHour`, `MicroRoentgenPerHour` |
| `CountDisplayUnit` | `Cps`, `Cpm` |

Labels: `DoseDisplayUnit::rate_label` / `dose_label` and `CountDisplayUnit::label`.

Encode/decode for alarm registers lives in `radiacode_lib::units`: `encode_dose_alarm`, `decode_dose_alarm`, `encode_count_alarm`, `decode_count_alarm`, `encode_dose_accum`, `decode_dose_accum`.

Real-time DataBuf conversion: `radiacode_lib::units::dose_display_from_rh`, `count_display_from_cps`.

## DataBufCursor / SeqGap

See [monitor.md](./monitor.md).

## SessionRestore

| Field | Type |
| --- | --- |
| `versions` | `DeviceVersions` |
| `spectrum_format_version` | `u32` |

## Error

`radiacode_lib::Error` is a flat, non-exhaustive enum. Protocol failures are variants of that enum, not a nested protocol error.

| Variant | When |
| --- | --- |
| `Timeout` | The device did not answer before the transport deadline |
| `ConnectionClosed` | The link was closed or lost |
| `ProtocolMismatch` | A frame or payload did not have the expected shape |
| `UnexpectedReturnCode` | Device return code other than success |
| `BufferUnderrun` | A payload ended before a field could be read |
| `VsfrBatchEmpty` | A batch register read reported no readable register |
| `TransportUnavailable` | Transport failure that is not a link loss |
| `DeviceNotFound` | No matching transport target |
| `IncompatibleFirmware` | Target < 4.8 on first connect |
| `MonitorDataPending` | DataBuf has no rate records yet |
| `UsbPermissionDenied` | Missing udev rule |
| `InvalidArgument` | A caller supplied a value the device cannot accept |

Helpers:

| Method | Meaning |
| --- | --- |
| `is_transient()` | Safe to retry (timeout, mismatch, underrun) |
| `is_connection_lost()` | Link dropped |
| `is_usb_permission_denied()` | Linux USB access |
| `is_timeout()` | `Error::Timeout` |

## Result alias

`type Result<T> = std::result::Result<T, Error>`

Used throughout `radiacode-lib` and the spectrum worker.
