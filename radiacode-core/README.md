# radiacode-core

High-level async client for RadiaCode devices. Sits above [`radiacode-protocol`](../radiacode-protocol/README.md) and below the transport crates ([`radiacode-usb`](../radiacode-usb/README.md), [`radiacode-bluetooth`](../radiacode-bluetooth/README.md)).

## Documentation

Full reference: [docs/core/README.md](../docs/core/README.md)

| Section | Topics |
| --- | --- |
| [session.md](../docs/core/session.md) | Connect, init, reconnect, timestamps |
| [api.md](../docs/core/api.md) | Complete `RadiaCode` method index |
| [monitor.md](../docs/core/monitor.md) | Live rates, polling, sequence gaps |
| [config.md](../docs/core/config.md) | Device settings and alarm limits |
| [discovery.md](../docs/core/discovery.md) | Scanning, endpoints, transports |
| [types.md](../docs/core/types.md) | Domain types, errors, units |

Wire-format detail lives in [docs/protocol/](../docs/protocol/README.md).

## Quick start

```toml
radiacode-core = { path = "../radiacode-core" }
radiacode-usb = { path = "../radiacode-usb" }
```

```rust
use radiacode_usb::{connect, scan_usb_devices};

#[tokio::main]
async fn main() -> radiacode_core::Result<()> {
    let devices = scan_usb_devices()?;
    let serial = devices[0].endpoint.address_label();
    let mut device = connect(serial).await?;

    let spectrum = device.spectrum().await?;
    let meta = device.metadata().await?;
    let total: u32 = spectrum.counts.iter().sum();
    println!("{} — {} channels, {} counts",
        meta.serial, spectrum.counts.len(), total);

    device.disconnect().await
}
```

Bluetooth:

```rust
use radiacode_bluetooth::{connect, scan_radiacode_devices};

let devices = scan_radiacode_devices().await?;
let address = devices[0].endpoint.address_label();
let mut device = connect(address).await?;
```

## Tests

```bash
cargo test -p radiacode-core
```

## License

AGPL-3.0-only — see [LICENSE](../LICENSE).
