# RadiaCode Wire Protocol

Wire-format notes for the RadiaCode protocol implemented by [radiacode-lib](https://github.com/ngxyzaudax/radiacode-lib). Field layouts mirror the device configuration INI (`VirtString::Configuration`, CP1251) and the SFR register directory (`VirtString::SfrFile`).

## Contents

| Document | Topics |
| --- | --- |
| [framing.md](./framing.md) | Request/response framing, sequence numbers, session init |
| [commands.md](./commands.md) | Command opcodes and virtual strings |
| [vsfr.md](./vsfr.md) | VSFR addresses, batch reads, unit scaling |
| [data-buf.md](./data-buf.md) | Monitor stream records, events, decoder robustness |
| [spectrum.md](./spectrum.md) | Histogram compression and calibration |
| [transports.md](./transports.md) | USB and Bluetooth LE constants |

## Quick reference

- Sequence range: `0x80..=0x9F` (wraps mod 32)
- Firmware gate: target version >= 4.8 unless explicitly ignored
- DataBuf tick resolution: 10 ms; wall time = `base_time + ticks * 10 ms` where `base_time = connect_time + 128 s`
- Configuration text encoding: CP1251
- Spectrum format version: from INI key `SpecFormatVersion=`

## Crate layout

| Module | Role |
| --- | --- |
| `protocol` | Frame builder, sequence counter, response assembler |
| `command` | Opcodes, virtual strings, VSFR catalog |
| `data_buf` | Monitor record decode and snapshot ranking |
| `spectrum` | Histogram decode |
| `catalog` | INI and SFR_FILE parsing, catalog validation |
| `rate_units` | Alarm and display unit conversions |
