<p align="center">
  <a href="https://github.com/ngxyzaudax/radiacode-spectrum-rust">
    <img src="./docs/logo.png" width="120" alt="Radiacode logo" />
  </a>
</p>

<h1 align="center">Radiacode Toolkit</h1>

<p align="center">
  Linux-first desktop software for <a href="https://www.radiacode.com/">RadiaCode</a> radiation detectors and spectrometers.
  <br />
  Connect over USB or Bluetooth LE — live readings, spectra, recordings, and device settings in one place.
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
  <a href="#screenshots"><strong>Screenshots</strong></a>
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
  <img src="./docs/demo/radiacode_demo.gif" width="900" alt="Radiacode Toolkit demo — live Monitor, Spectrum, Spectrogram, Analysis, and Settings" />
</p>

<p align="center">
  <a href="./docs/demo/radiacode_demo.webm">Full demo video (WebM, ~3:40)</a>
  ·
  <a href="./docs/README.md">Documentation index</a>
</p>

---

## Screenshots

Detailed notes for each tab live under [`docs/`](./docs/README.md).

### [Monitor](./docs/monitor/README.md)

<p align="center">
  <a href="./docs/monitor/README.md"><img src="./docs/monitor/screenshot.png" width="900" alt="Monitor tab" /></a>
</p>

Live dose rate, count rate, and session dose trends with alarm thresholds.

### [Spectrum](./docs/spectrum/README.md)

<p align="center">
  <a href="./docs/spectrum/README.md"><img src="./docs/spectrum/screenshot.png" width="900" alt="Spectrum tab" /></a>
</p>

Live 1024-channel energy histogram with calibrated keV axis.

### [Spectrogram](./docs/spectrogram/README.md)

<p align="center">
  <a href="./docs/spectrogram/README.md"><img src="./docs/spectrogram/screenshot.png" width="900" alt="Spectrogram tab" /></a>
</p>

Time–energy waterfall, recording transport, and `.rcspg` library.

### [Analysis](./docs/analysis/README.md)

<p align="center">
  <a href="./docs/analysis/README.md"><img src="./docs/analysis/screenshot.png" width="900" alt="Analysis tab" /></a>
</p>

Offline comparison of saved spectra with optional background subtraction.

### [Settings](./docs/settings/README.md)

<p align="center">
  <a href="./docs/settings/README.md"><img src="./docs/settings/screenshot.png" width="900" alt="Settings tab" /></a>
</p>

Device configuration: units, alarms, screen, and signal feedback.

---

## Quick start

### Requirements

- Rust **1.85+**
- Linux with USB and/or Bluetooth as needed
- Build dependencies: `libusb` headers and OpenGL/EGL for the GUI

### Build and run

```bash
git clone git@github.com:ngxyzaudax/radiacode-spectrum-rust.git
cd radiacode-spectrum-rust
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

---

## Architecture

Shared Rust crates sit behind the desktop app and handle protocol, transport, and device configuration.

| Crate | Role |
| --- | --- |
| `radiacode-spectrum` | Desktop application (egui) |
| `radiacode-core` | Device protocol, spectra, alarms, configuration |
| `radiacode-usb` | USB transport |
| `radiacode-bluetooth` | Bluetooth LE transport |

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
