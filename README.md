<p align="center">
  <a href="https://github.com/ngxyzaudax/radiacode-toolkit">
    <img src="./docs/logo.png" width="120" alt="Radiacode logo" />
  </a>
</p>

<h1 align="center">Radiacode Toolkit</h1>

<p align="center">
  Linux-first desktop software for <a href="https://www.radiacode.com/">RadiaCode</a> radiation detectors and spectrometers.
  <br />
  Connect over USB or Bluetooth LE — live readings, spectra, recordings, nuclide identification, and device settings in one place.
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.85%2B-orange?style=flat-square&logo=rust&logoColor=white" alt="Rust 1.85+" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0-blue?style=flat-square" alt="License AGPL-3.0" /></a>
  <img src="https://img.shields.io/badge/platform-Linux-1793D1?style=flat-square&logo=linux&logoColor=white" alt="Platform Linux" />
  <img src="https://img.shields.io/badge/GUI-egui%20%2B%20eframe-646464?style=flat-square" alt="GUI egui + eframe" />
</p>

<p align="center">
  <a href="#demo"><strong>Demo</strong></a>
  ·
  <a href="./docs/README.md"><strong>Docs</strong></a>
  ·
  <a href="./docs/PROTOCOL.md"><strong>Protocol</strong></a>
  ·
  <a href="./docs/NUCLIDES.md"><strong>Nuclides</strong></a>
  ·
  <a href="#quick-start"><strong>Quick start</strong></a>
  ·
  <a href="#architecture"><strong>Architecture</strong></a>
  ·
  <a href="#usb-access-on-linux"><strong>USB setup</strong></a>
</p>

---

## Demo

<p align="center">
  <img src="./docs/demo/radiacode_demo.gif" width="900" alt="Radiacode Toolkit — application tour" />
</p>

| Tab | What it does |
| --- | --- |
| [Device](./docs/device/README.md) | USB / Bluetooth discovery, connect, and disconnect |
| [Monitor](./docs/monitor/README.md) | Live dose rate, count rate, and session dose |
| [Spectrum](./docs/spectrum/README.md) | Live 1024-channel energy histogram with peak detection |
| [Spectrogram](./docs/spectrogram/README.md) | Time–energy waterfall and recordings |
| [Analysis](./docs/analysis/README.md) | Offline comparison of saved spectra |
| [Catalogue](./docs/catalogue/README.md) | Nuclide reference, decay chains, and synthetic previews |
| [Settings](./docs/settings/README.md) | Device and application configuration |

Per-tab demos and notes: [`docs/`](./docs/README.md). Wire format: [`docs/PROTOCOL.md`](./docs/PROTOCOL.md). Nuclear data: [`docs/NUCLIDES.md`](./docs/NUCLIDES.md).

---

## Quick start

### Requirements

- Rust **1.85+**
- Linux with USB and/or Bluetooth as needed
- Build dependencies: `libusb` headers and OpenGL/EGL for the GUI

On Debian/Ubuntu:

```bash
sudo apt-get install libudev-dev libusb-1.0-0-dev pkg-config \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev libssl-dev libdbus-1-dev
```

### Build and run

```bash
git clone git@github.com:ngxyzaudax/radiacode-toolkit.git
cd radiacode-toolkit
cargo build --release -p radiacode-spectrum
./target/release/radiacode-spectrum
```

Install to your PATH:

```bash
cargo install --path radiacode-spectrum
```

Optional desktop entry:

```bash
cp radiacode-spectrum/radiacode-spectrum.desktop ~/.local/share/applications/
```

### Tests

```bash
cargo test --workspace
```

---

## Architecture

Shared Rust crates sit behind the desktop app and handle protocol, transport, nuclear data, and device configuration.

| Crate | Role |
| --- | --- |
| [`radiacode-spectrum`](./radiacode-spectrum/README.md) | Desktop application (egui) |
| [`radiacode-nuclides`](./radiacode-nuclides/README.md) | Bundled nuclide catalogue, decay chains, peak matching |
| [`radiacode-protocol`](./radiacode-protocol/README.md) | Wire protocol — framing, opcodes, payload decoders, `Transport` trait ([reference](./docs/PROTOCOL.md)) |
| [`radiacode-core`](./radiacode-core/README.md) | Device client (`RadiaCode`) and domain models |
| [`radiacode-usb`](./radiacode-usb/README.md) | USB transport |
| [`radiacode-bluetooth`](./radiacode-bluetooth/README.md) | Bluetooth LE transport |

---

## USB access on Linux

RadiaCode USB devices use vendor `0483` / product `f123`. Without a udev rule, the app may not be able to open the device.

The app can install a rule when prompted. To install manually:

```bash
sudo cp radiacode.rules /etc/udev/rules.d/99-radiacode.rules
sudo udevadm control --reload
sudo udevadm trigger
```

Unplug and replug the detector after installing the rule.

---

<p align="center">
  <sub>Built for RadiaCode RC-1xx detectors · Linux-first · See <a href="LICENSE">LICENSE</a></sub>
</p>
