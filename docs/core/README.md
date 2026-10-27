# Device client reference

High-level device client and domain models for RadiaCode RC-1xx detectors. The implementation is [radiacode-lib](https://github.com/ngxyzaudax/radiacode-lib); this toolkit depends on that crate and keeps the GUI, nuclide catalogue, and spectrum analysis.

## Architecture

```text
radiacode-spectrum (GUI, nuclides, analysis)
        │
   radiacode-lib  ← protocol, USB, Bluetooth, RadiaCode client
```

`RadiaCode` owns a `Box<dyn Transport>`, a command sequence counter, cached firmware info, parsed SFR catalog entries, and a DataBuf base timestamp.

## Contents

| Document | Topics |
| --- | --- |
| [session.md](./session.md) | `open`, initialization, reconnect, wall-clock timestamps |
| [api.md](./api.md) | Full public API grouped by concern |
| [monitor.md](./monitor.md) | Monitor polling, live rates, status, cursor |
| [config.md](./config.md) | `DeviceConfig`, alarms, settings writes |
| [discovery.md](./discovery.md) | Device discovery and transport wiring |
| [types.md](./types.md) | Structs, enums, errors, display units |

## Related docs

- Wire protocol: [protocol/README.md](../protocol/README.md)
- Nuclide catalogue: [NUCLIDES.md](../NUCLIDES.md)

## Where the types live

`radiacode-lib` exports the session types from the crate root (`RadiaCode`, `Spectrum`, `DeviceEndpoint`, …). Registers and opcodes are under `radiacode_lib::protocol`, settings under `radiacode_lib::config`, and the transports under `radiacode_lib::usb` and `radiacode_lib::bluetooth`.

Low-level framing and opcode tables are documented under [protocol/](../protocol/README.md).
