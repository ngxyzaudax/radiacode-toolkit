# radiacode-bluetooth

Bluetooth LE transport for RadiaCode detectors.

## Features

- BLE scan and advertisement parsing (model, serial, RSSI)
- `BleTransport` implementing `radiacode_protocol::Transport`
- Linux RSSI via BlueZ where available

## Usage

```toml
radiacode-bluetooth = { path = "../radiacode-bluetooth" }
```

Requires a running Bluetooth adapter and paired/visible RadiaCode device.

Pair with `radiacode_core::RadiaCode::connect` for the full client.

## Dependencies

Uses [btleplug](https://github.com/deviceplug/btleplug) for cross-platform BLE. On Linux, optional BlueZ management crates support RSSI reads.

## License

AGPL-3.0-only — see [LICENSE](../LICENSE).
