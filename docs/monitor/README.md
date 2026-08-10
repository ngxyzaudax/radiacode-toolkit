# Monitor

<p align="center">
  <img src="./screenshot.png" width="900" alt="Monitor tab — live dose and count rate trends" />
</p>

The Monitor view is the primary operational dashboard for continuous radiation surveillance. It streams dose rate, count rate, and session-accumulated dose from the detector and plots each quantity as a rolling time series with warn and danger thresholds overlaid as reference lines. Numeric readouts and alarm configuration live in the sidebar so you can observe trends and adjust limits without switching context.

## Features

- Rolling **120 s** windows for dose rate and count rate; cumulative dose plotted over the full session
- Live readouts: dose rate, count rate, accumulated dose, session elapsed time (device-reported units)
- Alarm cards for dose rate, count rate, and accumulated dose — warn/danger thresholds, audible and visual toggles, load/save to device
- Filled or outline chart rendering; confirmed reset of accumulated dose on the detector

## Related

- [Spectrum](../spectrum/README.md)
- [Settings](../settings/README.md)
- [Docs index](../README.md)
