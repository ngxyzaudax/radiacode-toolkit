# radiacode-core Reference

High-level device client and domain models for RadiaCode RC-1xx detectors.

## Architecture

```text
radiacode-spectrum (GUI)
        │
radiacode-usb / radiacode-bluetooth  ← Transport implementations
        │
   radiacode-core  ← this crate
        │
 radiacode-protocol  ← wire format
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

## Re-exports

`radiacode-core` re-exports commonly used protocol types (`Spectrum`, `Transport`, `VirtSfr`, `decode_data_buf`, …) so application code can depend on a single crate when appropriate.

Low-level framing and opcode tables are documented under [protocol/](../protocol/README.md), not duplicated here.
