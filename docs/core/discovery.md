# Discovery and transports

## Discovery types

| Type | Role |
| --- | --- |
| `DiscoveredDevice` | One scan result with label, serial, model, RSSI |
| `DeviceEndpoint` | `Bluetooth { address }` or `Usb { serial }` |
| `TransportKind` | `Bluetooth` or `Usb` |

`DiscoveredDevice::display_label()` prefers advertisement label, then model, serial, or `"RadiaCode"`.

## Scanning

| Module | Function | Returns |
| --- | --- | --- |
| `radiacode_lib::usb` | `scan_usb_devices().await` | `Vec<DiscoveredDevice>` |
| `radiacode_lib::bluetooth` | `scan_bluetooth_devices(duration).await` | `Vec<DiscoveredDevice>` |

Merge USB and Bluetooth results:

```rust
use radiacode_lib::discovery::merge_discovered;

let all = merge_discovered(usb_devices, ble_devices);
```

Bluetooth entries whose serial matches an already-seen USB device are dropped.

## Connecting

### USB

```rust
use radiacode_lib::usb::scan_usb_devices;
use radiacode_lib::{OpenOptions, connect};

let devices = scan_usb_devices().await?;
let device = connect(&devices[0].endpoint, OpenOptions::default()).await?;
```

Requires udev rule on Linux — see [README.md](../../README.md#usb-access-on-linux).

Permission errors surface as `Error::UsbPermissionDenied`.

### Bluetooth

```rust
use radiacode_lib::bluetooth::scan_bluetooth_devices;
use radiacode_lib::{OpenOptions, connect};

let devices = scan_bluetooth_devices(std::time::Duration::from_secs(5)).await?;
let device = connect(&devices[0].endpoint, OpenOptions::default()).await?;
```

On macOS, `address` is the CoreBluetooth UUID, not a MAC address.

## Transport trait

Defined in `radiacode-lib`, implemented by `UsbTransport` and `BluetoothTransport`:

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
use radiacode_lib::discovery::resolve_usb_endpoint;

let endpoint = resolve_usb_endpoint(&devices, &preferred_endpoint);
```

Matches by USB serial key or user-visible serial string; falls back to the first USB device.

## Reconnect

After an unexpected disconnect, avoid full firmware probing by passing the saved session:

```rust
use radiacode_lib::bluetooth::BluetoothTransport;
use radiacode_lib::{OpenOptions, RadiaCode, connect};

let restore = device.session_restore().expect("initialized");
let options = OpenOptions::default().restore(restore);

// USB:
let device = connect(&endpoint, options).await?;

// Bluetooth (rescan, then restore):
let transport = BluetoothTransport::connect_fresh(address).await?;
let device = RadiaCode::open(Box::new(transport), options).await?;
```

Reconnect skips `GetVersion` and Configuration reads but still runs the exchange handshake, clock sync, and SFR cache reload. `connect` uses the cached Bluetooth peripheral; `BluetoothTransport::connect_fresh` scans again.

## Advertisement parsing

BLE scan names like `RC-110-006806` are parsed by:

- `serial_from_advertisement(name)` → `"RC-110-006806"`
- `model_from_advertisement(name)` → `"RC-110"`
- `model_from_serial(serial)` → model prefix from serial text

Used by the GUI device list and `DeviceMetadata`.
