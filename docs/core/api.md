# RadiaCode API reference

All methods are on `RadiaCode` unless noted. Free functions have matching inherent methods unless stated otherwise.

## Identity and firmware

| Method | Returns | Wire |
| --- | --- | --- |
| `serial_number()` | User-visible serial (ASCII) | `VirtString::SerialNumber` |
| `hw_serial_number()` | Hardware serial (hex groups) | `GetSerial` |
| `fw_version()` | `DeviceVersions` | `GetVersion` |
| `configuration()` | Device INI text (CP1251) | `VirtString::Configuration` |
| `metadata()` | `DeviceMetadata` | Serial + cached versions + energy calib |

## Spectrum and calibration

| Method | Returns | Wire |
| --- | --- | --- |
| `spectrum()` | Live `Spectrum` | `VirtString::Spectrum` |
| `spectrum_accum()` | Accumulated `Spectrum` | `VirtString::SpecAccum` |
| `spectrum_diff()` | Differential `Spectrum` | `VirtString::SpecDiff` |
| `energy_calib()` | `[a0, a1, a2]` | `VirtString::EnergyCalib` |
| `set_energy_calib([a0,a1,a2])` | — | `WrVirtString(EnergyCalib)` |
| `spectrum_reset()` | — | `WrVirtString(Spectrum, len=0)` |
| `spectrum_format_version()` | `u32` | Cached from Configuration |

## Monitor and status

| Method | Returns | Notes |
| --- | --- | --- |
| `live_rates(units)` | `LiveRates` | Latest snapshot rate with display units |
| `poll_monitor(units, cursor, refresh_rssi)` | `(MonitorPollSample, DeviceStatus)` | RealTimeData series + status |
| `device_status(refresh_rssi)` | `DeviceStatus` | One-shot status read |

See [monitor.md](./monitor.md) for poll semantics.

## Alarms

| Method | Returns | Notes |
| --- | --- | --- |
| `alarm_limits()` | `AlarmLimits` | Batch VSFR read with unit conversion |
| `set_alarm_limits(update)` | — | Partial update via `AlarmLimitsUpdate` |

## Device configuration

| Method | Returns | Notes |
| --- | --- | --- |
| `load_device_config()` | `DeviceConfig` | Alarms + UI settings in one batch |
| `apply_device_config(config)` | — | Batch VSFR write |
| `sync_device_clock()` | — | `SetTime` + `DeviceTime = 0` |

See [config.md](./config.md) for struct fields.

## Settings (individual writes)

| Method | VSFR |
| --- | --- |
| `set_display_brightness(0..=9)` | `DispBrt` |
| `set_sound_on(bool)` | `SoundOn` |
| `set_vibro_on(bool)` | `VibroOn` |
| `set_language(DeviceLanguage)` | `DeviceLang` |
| `set_device_on(bool)` | `DeviceOn` |
| `dose_reset()` | `DoseReset` |

## Diagnostics

| Method | Returns | Wire |
| --- | --- | --- |
| `status()` | Status flags `u32` | `GetStatus` |
| `fw_signature()` | `FirmwareSignature` | `FwSignature` |
| `text_message()` | Device text (ASCII) | `VirtString::TextMessage` |
| `sfr_file_text()` | SFR directory text | `VirtString::SfrFile` |

## VSFR and virtual strings

| Method | Notes |
| --- | --- |
| `read_virt_string(id)` | Raw payload after retcode/flen handling |
| `read_vsfr_u32(id)` | Single register |
| `read_vsfr_f32(id)` | Reinterpret u32 as f32 |
| `read_vsfr_optional(id)` | `None` when retcode is 0 |
| `write_vsfr(id, data)` | Fails if retcode ≠ 1 |
| `write_vsfr_optional(id, data)` | Returns `false` when retcode is 0 |
| `read_vsfr_batch(ids)` | Typed extraction via SFR catalog |
| `write_vsfr_batch(pairs)` | Batch write with validity mask check |

Free functions:

| Function | Purpose |
| --- | --- |
| `read_vsfr_batch_raw(device, ids)` | Raw u32 words without extraction |
| `extract_vsfr_value(device, id, raw)` | Apply catalog-driven extraction to one word |

## Session introspection

| Method | Returns |
| --- | --- |
| `session_restore()` | `Option<SessionRestore>` |
| `base_time()` | DataBuf wall-clock anchor |
| `sfr_entries()` | Parsed `SfrCatalogEntry` list |
| `wall_clock(ticks)` | `SystemTime` for a DataBuf tick |

## Standalone helpers

| Function | Module |
| --- | --- |
| `merge_status(carry, fresh)` | Status merge for sticky battery/temp/RSSI |
| `merge_discovered(usb, ble)` | Dedupe scan results by serial |
| `resolve_usb_endpoint(devices, preferred)` | Pick USB device when multiple attached |
| `model_from_serial`, `serial_from_advertisement`, `model_from_advertisement` | BLE advertisement parsing |
| `dose_unit_label`, `count_unit_label`, `dose_accum_unit_label` | Display strings |

Protocol decode helpers (`decode_data_buf`, `latest_snapshot`, `channel_to_energy`, …) are re-exported from `radiacode-protocol`.
