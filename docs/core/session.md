# Session lifecycle

## Opening a connection

```rust
RadiaCode::open(
    transport: Box<dyn Transport>,
    ignore_firmware_check: bool,
    restore: Option<&SessionRestore>,
) -> Result<RadiaCode>
```

Transport crates provide convenience wrappers:

| Crate | Function | Notes |
| --- | --- | --- |
| `radiacode-usb` | `connect(serial)` | First connect, full init |
| `radiacode-usb` | `reconnect_session(serial, restore)` | Fast reconnect |
| `radiacode-bluetooth` | `connect(address)` | Same as USB `connect` |
| `radiacode-bluetooth` | `reconnect_session(address, restore)` | Same as USB reconnect |

## Initialization sequence

Every open (including reconnect) runs:

1. `SetExchange` with `01 ff 12 ff`
2. Drain transport, 250 ms settle
3. `SetTime` to host local clock
4. Write `DeviceTime = 0`
5. Set `base_time = now + 128 s`
6. Parse and cache `SfrFile` entries

**First connect only** (when `restore` is `None`):

7. `GetVersion` — cached; reject firmware < 4.8 unless `ignore_firmware_check`
8. Read `Configuration` (CP1251) — parse `SpecFormatVersion`
9. Validate static VSFR catalog against device `SfrFile` (warn on drift)

**Reconnect** (when `restore` is `Some`):

Steps 7–9 are skipped. Cached `DeviceVersions` and `spectrum_format_version` are taken from `SessionRestore`.

## Retry policy

| Mode | Attempts | Delay between retries |
| --- | --- | --- |
| First connect | 3 | 1500 ms |
| Reconnect | 2 | 3000 ms |

Transient protocol errors (timeout, buffer underrun) retry with the same transport handle. Non-transient errors disconnect and return immediately.

## SessionRestore

Capture before disconnect for fast reconnect:

```rust
if let Some(restore) = device.session_restore() {
    // persist restore.versions and restore.spectrum_format_version
}
```

Fields:

| Field | Purpose |
| --- | --- |
| `versions` | Boot and target firmware from first connect |
| `spectrum_format_version` | Spectrum compression format (0 or 1) |

On reconnect, pass `Some(&restore)` to `RadiaCode::open` or use `reconnect_session` from a transport crate.

## Timestamps

DataBuf record headers carry relative ticks (10 ms resolution). Convert to wall time:

```rust
let wall = device.wall_clock(record.header.ts);
```

`base_time()` returns the anchor set at init (`connect_time + 128 s` after `DeviceTime = 0`).

## Disconnect

```rust
device.disconnect().await?;
```

Consumes `RadiaCode` and calls `Transport::disconnect`.

## Low-level access

`execute_raw(command, args)` sends any opcode and returns the post-echo body. Prefer typed methods when available.

`drain_transport()` clears stale inbound data before sensitive reads (used internally for `fw_version` retries).

## RSSI

| Method | Behavior |
| --- | --- |
| `rssi_dbm()` | Cached link RSSI from transport |
| `sample_rssi_dbm()` | Fresh sample (Bluetooth); used during monitor poll when `refresh_rssi = true` |

USB transport always returns `None`.
