# Discovery and transports

## Discovery types

| Type | Role |
| --- | --- |
| `DiscoveredDevice` | One scan result with label, serial, model, RSSI |
| `DeviceEndpoint` | `Bluetooth { address }` or `Usb { serial }` |
| `TransportKind` | `Bluetooth` or `Usb` |

`DiscoveredDevice::display_label()` prefers advertisement label, then model, serial, or `"RadiaCode"`.

## Scanning

| Crate | Function | Returns |
| --- | --- | --- |
| `radiacode-usb` | `scan_usb_devices()` | `Vec<DiscoveredDevice>` |
| `radiacode-bluetooth` | `scan_radiacode_devices()` | `Vec<DiscoveredDevice>` |

Merge USB and Bluetooth results:

```rust
use radiacode_core::merge_discovered;

let all = merge_discovered(usb_devices, ble_devices);
```

Bluetooth entries whose serial matches an already-seen USB device are dropped.

## Connecting

### USB

```rust
use radiacode_usb::{connect, scan_usb_devices};

let devices = scan_usb_devices()?;
let serial = devices[0].endpoint.address_label();
let device = connect(serial).await?;
```

Requires udev rule on Linux — see [README.md](../../README.md#usb-access-on-linux).

Permission errors surface as `Error::UsbPermissionDenied`.

### Bluetooth

```rust
use radiacode_bluetooth::{connect, scan_radiacode_devices};

let devices = scan_radiacode_devices().await?;
let address = devices[0].endpoint.address_label();
let device = connect(address).await?;
```

On macOS, `address` is the CoreBluetooth UUID, not a MAC address.

## Transport trait

Defined in `radiacode-protocol`, implemented by `UsbTransport` and `BluetoothTransport`:

| Method | Purpose |
| --- | --- |
| `execute(request)` | Send framed request, return reassembled payload |
| `drain_link()` | Discard stale inbound data |
| `disconnect()` | Release link |
| `link_rssi_dbm()` | Cached RSSI |
| `sample_link_rssi_dbm()` | Fresh RSSI sample |

Custom transports can be passed directly to `RadiaCode::open`.

## Multiple USB devices

When more than one detector is attached:

```rust
use radiacode_core::resolve_usb_endpoint;

let endpoint = resolve_usb_endpoint(&devices, &preferred_endpoint);
```

Matches by USB serial key or user-visible serial string; falls back to the first USB device.

## Reconnect

After an unexpected disconnect, avoid full firmware probing:

```rust
let restore = device.session_restore().expect("initialized");

// USB:
radiacode_usb::reconnect_session(serial, &restore).await?;

// Bluetooth:
radiacode_bluetooth::reconnect_session(address, &restore).await?;
```

Reconnect skips `GetVersion` and Configuration reads but still runs the exchange handshake, clock sync, and SFR cache reload.

## Advertisement parsing

BLE scan names like `RC-110-006806` are parsed by:

- `serial_from_advertisement(name)` → `"RC-110-006806"`
- `model_from_advertisement(name)` → `"RC-110"`
- `model_from_serial(serial)` → model prefix from serial text

Used by the GUI device list and `DeviceMetadata`.
