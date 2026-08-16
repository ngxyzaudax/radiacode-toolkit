# radiacode-spectrum

Desktop application for RadiaCode detectors — live monitoring, spectroscopy, recordings, nuclide identification, and device configuration.

Built with [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe).

## Run

From the workspace root:

```bash
cargo run -p radiacode-spectrum
```

Release build:

```bash
cargo build --release -p radiacode-spectrum
./target/release/radiacode-spectrum
```

Install:

```bash
cargo install --path radiacode-spectrum
```

## Tabs

| Tab | Module | Docs |
| --- | --- | --- |
| Device | `src/device/` | [docs/device](../docs/device/README.md) |
| Monitor | `src/monitor/` | [docs/monitor](../docs/monitor/README.md) |
| Spectrum | `src/ui_plot.rs`, `src/peaks/` | [docs/spectrum](../docs/spectrum/README.md) |
| Spectrogram | `src/spectrogram/` | [docs/spectrogram](../docs/spectrogram/README.md) |
| Analysis | `src/analysis/` | [docs/analysis](../docs/analysis/README.md) |
| Catalogue | `src/catalogue/` | [docs/catalogue](../docs/catalogue/README.md) |
| Settings | `src/settings/` | [docs/settings](../docs/settings/README.md) |

## Key modules

| Path | Role |
| --- | --- |
| `src/app.rs` | Application shell, worker events, tab routing |
| `src/worker.rs` | Background Tokio worker for device I/O |
| `src/peaks/` | SNIP continuum, matched-filter detection, Poisson scoring |
| `src/peak_overlay/` | Shared peak markers and nuclide chips |
| `src/identify.rs` | Detection + catalogue matching pipeline |
| `src/app_config.rs` | Persistent application settings |

Configuration is stored under the XDG config directory (`radiacode-spectrum/app_config.json`).

## Dependencies

- `radiacode-core` — device client
- `radiacode-nuclides` — catalogue and peak matching
- `radiacode-usb` / `radiacode-bluetooth` — transports

## Tests

```bash
cargo test -p radiacode-spectrum
```

Fixture spectra live in `data/spectra/` for peak-detection regression tests.

## License

AGPL-3.0-only — see [LICENSE](../LICENSE).
