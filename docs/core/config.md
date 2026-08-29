# Device configuration

## DeviceConfig

Loaded and applied atomically via batch VSFR I/O:

```rust
let config = device.load_device_config().await?;
device.apply_device_config(&config).await?;
```

| Field | VSFR source | Notes |
| --- | --- | --- |
| `alarms` | CR/DR/DS level registers + unit flags | See `AlarmLimits` |
| `alarm_mode` | `AlarmMode` | Once or Continuous |
| `brightness` | `DispBrt` | 0..9 |
| `backlight_off` | `DispOffTime` | 5 s … 5 min |
| `display_dir` | `DispDir` | Auto, Right, Left |
| `sound_on` | `SoundOn` | |
| `vibro_on` | `VibroOn` | |
| `sound_ctrl` | `SoundCtrl` | `SignalFlags` bitmask |
| `vibro_ctrl` | `VibroCtrl` | `SignalFlags` without clicks/connection/power |
| `leds_on` | `LedsOn` or `DeviceCtrl` bit 3 | Depends on hardware |
| `leds_supported` | Derived from SFR_FILE | |
| `leds_uses_device_ctrl` | No `LedsOn` in SFR_FILE | Legacy light toggle |

### BacklightOffTime wire values

| Enum | Raw |
| --- | --- |
| Sec5 | 0 |
| Sec10 | 1 |
| Sec15 | 2 |
| Sec30 | 3 |
| Min2 | 4 |
| Min5 | 5 |

### SignalFlags

Sound and vibration event masks. Bits 0–12 cover buttons, clicks, dose/count alarms, connection, and power events. Vibration control strips clicks, connection, and power via `vibro_events()` before write.

## AlarmLimits

Read/write separately from full config when only thresholds change:

```rust
let limits = device.alarm_limits().await?;

device.set_alarm_limits(&AlarmLimitsUpdate {
    l1_count_rate: Some(20.0),
    dose_unit: Some(DoseDisplayUnit::MicroSievertPerHour),
    ..Default::default()
}).await?;
```

| Field | Display unit | Wire storage |
| --- | --- | --- |
| `l1/l2_count_rate` | cps or cpm | cp10s register |
| `l1/l2_dose_rate` | µSv/h or µR/h | µR/h register |
| `l1/l2_dose` | µSv or µR accumulated | µR register |

`AlarmLimitsUpdate` uses `Option` fields — only supplied fields are written. Empty update is a no-op.

Compared to the Python [cdump/radiacode](https://github.com/cdump/radiacode) library: accumulated dose alarm values here are in **micro-units** (µSv/µR), not base units divided by 1e6.

## Individual settings

For single-field changes without reloading full config:

```rust
device.set_display_brightness(5).await?;
device.set_sound_on(true).await?;
device.set_language(DeviceLanguage::English).await?;
device.set_device_on(true).await?;
device.dose_reset().await?;
device.spectrum_reset().await?;
```

## Clock sync

```rust
device.sync_device_clock().await?;
```

Sets host local time on the device and resets `DeviceTime` to 0. Does not update `base_time`; reconnect or re-open for a fresh anchor.

## VSFR batch extraction

`load_device_config` and `alarm_limits` use `read_vsfr_batch`, which applies SFR-catalog-driven sub-word extraction. Sub-byte registers may arrive in the low or high byte of the returned u32; see [protocol/vsfr.md](../protocol/vsfr.md).

Use `read_vsfr_batch_raw` when debugging register encoding:

```bash
cargo run -p radiacode-usb --example probe_settings
```

## Energy calibration write

```rust
let coef = device.energy_calib().await?;
device.set_energy_calib(coef).await?;  // write only after intentional change
```

Preserve existing coefficients before modifying. Calibration affects channel→energy mapping used by spectrum analysis.
