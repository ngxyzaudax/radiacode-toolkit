# Radiacode Toolkit

Linux-first desktop software for [RadiaCode](https://www.radiacode.com/) radiation detectors and spectrometers (RC-1xx series). Connect over USB or Bluetooth LE, view live readings, capture spectra, and adjust device settings from one app.

Developed and tested on Linux. USB permissions, Bluetooth pairing, and desktop integration assume a typical Linux environment (udev, BlueZ, X11/Wayland). Other platforms may compile but are not supported targets.

## Radiacode Spectrum

The main application (`radiacode-spectrum`) provides:

- **Device** — connect, disconnect, and view device status
- **Monitor** — live dose and count rates, accumulated dose, trend charts, and alarm controls
- **Spectrum** — energy histogram with smoothing and scale options
- **Spectrogram** — time–energy waterfall, recording, and playback
- **Analysis** — compare saved recordings against a background spectrum
- **Settings** — alarm thresholds, units, display options, signal preferences, and app behavior

Shared Rust crates handle protocol, USB, Bluetooth, and device configuration behind the UI.

| Crate | Role |
| --- | --- |
| `radiacode-core` | Device protocol, spectra, alarms, configuration |
| `radiacode-usb` | USB transport |
| `radiacode-bluetooth` | Bluetooth LE transport |
| `radiacode-spectrum` | Desktop application |

## Requirements

- Rust 1.85+
- Linux with USB and/or Bluetooth as needed
- Build dependencies: `libusb` headers and OpenGL/EGL for the GUI

## Build and run

```bash
git clone <repo-url>
cd radiacode
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

## USB access on Linux

RadiaCode USB devices use vendor `0483` / product `f123`. Without a udev rule, the app may not be able to open the device.

The app can install a rule when prompted. To install manually:

```bash
sudo cp radiacode.rules /etc/udev/rules.d/99-radiacode.rules
sudo udevadm control --reload
sudo udevadm trigger
```

Unplug and replug the detector after installing the rule.

## License

See [LICENSE](LICENSE).
