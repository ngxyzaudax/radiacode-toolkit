# Spectrogram

<p align="center">
  <img src="./demo.gif" width="900" alt="Spectrogram tab — time–energy waterfall with recording library" />
</p>

The Spectrogram view captures how the energy spectrum changes over time. Each row is a snapshot taken at a configurable interval; rows stack into a time–energy waterfall for spotting drift, transient peaks, or environmental changes across long runs. A built-in library stores sessions on disk, supports search, and handles import/export of `.rcspg` recordings for offline review.

## Features

- Time–energy waterfall with configurable capture interval and row limit
- **Spectrum preview strip** above the heatmap — whole-series sum aligned to visible energy columns, with **Lin/Log** scale and **Peaks** toggles in the left gutter
- Colormap selection (Viridis, Inferno, Turbo); auto brightness, grid, count-rate overlays
- Recording transport: record, pause, play, stop; session info on preview-strip hover
- Searchable recording library with metadata, comments, import, export, and replay
- **Peak detection** on the collapsed spectrum (all rows summed); markers and preview strip share column-aligned energy mapping
- Identified nuclide chips shared with the Spectrum overlay

## Related

- [Spectrum](../spectrum/README.md)
- [Analysis](../analysis/README.md)
- [Docs index](../README.md)
