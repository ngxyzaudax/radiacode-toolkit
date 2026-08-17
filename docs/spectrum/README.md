# Spectrum

<p align="center">
  <img src="./demo.gif" width="900" alt="Spectrum tab — live energy histogram" />
</p>

The Spectrum view displays the live 1024-channel energy histogram from the detector. It is intended for peak identification, calibration verification, and monitoring how counts accumulate in each channel over time. The energy axis is derived from the on-device calibration polynomial; live time, total counts, channel count, and calibration coefficients appear in the toolbar row alongside scale, peak detection, and smoothing controls.

## Features

- 1024-channel histogram with calibrated energy axis (keV)
- Linear or logarithmic Y scale; adjustable smoothing window (channels)
- Filled or outline chart style; reset accumulation
- Toolbar row: `hh:mm:ss` live time, total counts, channel count, calibration formula, linear/log scale, chart style, peak detection, and smoothing
- **Peak detection** — SNIP continuum removal, matched-filter peaks, Poisson significance; toggle on the toolbar
- **Sticky peak cursor** — with peak detection on, the crosshair snaps to the nearest peak line within 12 px and shows nuclide, energy, and net area in the hover readout
- Scroll wheel zooms the energy axis at the pointer (scroll up = zoom in); drag pans; double-click resets to full range
- Identified nuclide chips on detected peaks; click to open the matching catalogue entry
- Resolution-aware matching using detector FWHM (Settings → Application → Peak detection and identification)

## Related

- [Catalogue](../catalogue/README.md)
- [Spectrogram](../spectrogram/README.md)
- [Compare](../compare/README.md)
- [NUCLIDES.md](../NUCLIDES.md)
- [Docs index](../README.md)
