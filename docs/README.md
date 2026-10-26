# Documentation

Screenshots and demo recordings for each main application tab, plus references for the wire protocol and nuclide catalogue.

## Demo

<p align="center">
  <img src="./demo/radiacode_demo.gif" width="900" alt="Radiacode Toolkit — application tour" />
</p>

Full application tour as a GIF.

## Application tabs

| Tab | Description |
| --- | --- |
| [Device](./device/README.md) | USB / Bluetooth discovery, connect, and disconnect |
| [Monitor](./monitor/README.md) | Live dose rate, count rate, and session dose trends |
| [Spectrum](./spectrum/README.md) | Live 1024-channel energy histogram with peak detection |
| [Spectrogram](./spectrogram/README.md) | Time–energy waterfall and recording library |
| [Compare](./compare/README.md) | Offline comparison of saved spectra |
| [Catalogue](./catalogue/README.md) | Nuclide reference, decay chains, and synthetic previews |
| [Settings](./settings/README.md) | Device and application configuration |

## References

| Document | Description |
| --- | --- |
| [PROTOCOL.md](./PROTOCOL.md) | RadiaCode wire format index ([full reference](./protocol/README.md)) |
| [CORE.md](./CORE.md) | Device client and domain models ([full reference](./core/README.md)) |
| [NUCLIDES.md](./NUCLIDES.md) | Catalogue data provenance, regeneration, and peak matching |

## Crate READMEs

| Crate | README |
| --- | --- |
| `radiacode-spectrum` | [../radiacode-spectrum/README.md](../radiacode-spectrum/README.md) |
| `radiacode-nuclides` | [../radiacode-nuclides/README.md](../radiacode-nuclides/README.md) |
| `radiacode-lib` | [github.com/ngxyzaudax/radiacode-lib](https://github.com/ngxyzaudax/radiacode-lib) |
