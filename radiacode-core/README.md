# radiacode-core

RadiaCode device client and shared domain models. Sits above `radiacode-protocol` and below the transport crates.

## Responsibilities

- **`RadiaCode`** — high-level async client for spectrum fetch, monitor polling, configuration read/write, and status
- **Discovery** — `DiscoveredDevice`, `DeviceEndpoint`, USB/Bluetooth merge helpers
- **Configuration** — alarm limits, display settings, signal flags, clock sync
- **Data buffer cursor** — incremental decode of monitor stream records with sequence-gap detection
- **Re-exports** — protocol types (`Spectrum`, `Transport`, `RealTimeRates`, …) for convenience

## Usage

Add to `Cargo.toml`:

```toml
radiacode-core = { path = "../radiacode-core" }
```

Typical flow with a transport implementation:

```rust
use radiacode_core::{RadiaCode, Transport};

async fn read_spectrum(transport: impl Transport) -> radiacode_core::Result<()> {
    let mut device = RadiaCode::connect(transport).await?;
    let spectrum = device.fetch_spectrum().await?;
    let _ = spectrum.total_counts();
    Ok(())
}
```

Use `radiacode-usb` or `radiacode-bluetooth` for concrete `Transport` implementations.

## Tests

```bash
cargo test -p radiacode-core
```

## License

AGPL-3.0-only — see [LICENSE](../LICENSE).
