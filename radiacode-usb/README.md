# radiacode-usb

USB transport for RadiaCode detectors (vendor `0483`, product `f123`).

## Features

- Device scan and `DiscoveredDevice` enumeration
- `UsbTransport` implementing `radiacode_protocol::Transport`
- Linux udev rule helper for unprivileged access

## Usage

```toml
radiacode-usb = { path = "../radiacode-usb" }
```

```rust
use radiacode_usb::{scan_usb_devices, UsbTransport};

let devices = scan_usb_devices()?;
```

Pair with `radiacode_core::RadiaCode::connect` for the full client.

## Linux setup

Install the udev rule from the workspace root — see [README](../README.md#usb-access-on-linux).

## Examples

```bash
cargo run -p radiacode-usb --example scan
```

## License

AGPL-3.0-only — see [LICENSE](../LICENSE).
