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
  <a href="#screenshots"><strong>Screenshots</strong></a>
  ·
  <a href="#features"><strong>Features</strong></a>
  ·
  <a href="#quick-start"><strong>Quick start</strong></a>
  ·
  <a href="#architecture"><strong>Architecture</strong></a>
  ·
  <a href="#usb-access-on-linux"><strong>USB setup</strong></a>
</p>

---

<p align="center">
  <img src="./docs/spectrogram_tab.png" width="900" alt="Spectrogram view — time–energy waterfall with recording library" />
</p>

<p align="center"><em>Spectrogram — capture, browse, and replay time–energy recordings</em></p>

---

## Screenshots

### Spectrum

<p align="center">
  <img src="./docs/spectrum_tab.png" width="900" alt="Spectrum tab — energy histogram with log scale and smoothing" />
</p>

<p align="center"><sub>Energy histogram with log scale, smoothing, and filled or outline charts</sub></p>

### Spectrogram

<p align="center">
  <img src="./docs/spectrogram_tab.png" width="900" alt="Spectrogram tab — waterfall view with recording library" />
</p>

<p align="center"><sub>Waterfall view, palette controls, and a searchable recording library</sub></p>

### Analysis

<p align="center">
  <img src="./docs/analysis_tab.png" width="900" alt="Analysis tab — overlay saved spectra against background" />
</p>

<p align="center"><sub>Overlay saved spectra, compare samples against background, subtract and smooth</sub></p>

---

## Features

<table>
  <tr>
    <td width="50%" valign="top">

### Live monitoring

Monitor dose rate, count rate, and accumulated dose with trend charts and alarm controls — all in one view alongside your session readouts.

### Spectrum analysis

Inspect the live energy histogram with linear or log scale, adjustable smoothing, and a filled or outline chart style.

    </td>
    <td width="50%" valign="top">

### Recording & playback

Capture spectrogram rows at a fixed interval, import `.rcspg` files, and browse your library with search and metadata at a glance.

### Device settings

Configure alarm thresholds, units, display options, and signal preferences from a dedicated settings workspace.

    </td>
  </tr>
</table>

Every tab shares a consistent dark UI, sidebar-driven controls, and transport layers for both USB and Bluetooth LE. Developed and tested on Linux.

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
