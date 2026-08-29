# Commands and virtual strings

## Command opcodes

| Opcode | Name | Purpose |
| --- | --- | --- |
| `0x0005` | GetStatus | Device status flags (`u32`) |
| `0x0007` | SetExchange | Session handshake |
| `0x000A` | GetVersion | Boot and target firmware versions |
| `0x000B` | GetSerial | Hardware serial (hex groups) |
| `0x0012` | FwImageGetInfo | Firmware image metadata |
| `0x0101` | FwSignature | Signature plus filename and id strings |
| `0x0807` | RdHwConfig | Hardware configuration block |
| `0x081C` | RdFlash | Raw flash read |
| `0x0824` | RdVirtSfr | Read one VSFR |
| `0x0825` | WrVirtSfr | Write one VSFR |
| `0x0826` | RdVirtString | Read virtual string |
| `0x0827` | WrVirtString | Write virtual string |
| `0x082A` | RdVirtSfrBatch | Batch VSFR read |
| `0x082B` | WrVirtSfrBatch | Batch VSFR write |
| `0x0A04` | SetTime | Set device local clock |

## Virtual strings

| ID | Name | Purpose |
| --- | --- | --- |
| `2` | Configuration | Device INI (CP1251) |
| `3` | FwDescriptor | Firmware descriptor text |
| `8` | SerialNumber | User-visible serial |
| `0x0F` | TextMessage | Device text message (ASCII) |
| `0xE0` | MemSnapshot | Memory snapshot blob |
| `0x100` | DataBuf | Ring buffer of monitor records |
| `0x101` | SfrFile | Self-describing VSFR directory |
| `0x200` | Spectrum | Live histogram |
| `0x202` | EnergyCalib | `[a0, a1, a2]` calibration |
| `0x205` | SpecAccum | Accumulated histogram |
| `0x206` | SpecDiff | Differential spectrum |
| `0x207` | SpecReset | Spectrum reset channel |

## GetVersion body

`boot_minor u16`, `boot_major u16`, boot date (u8-length ASCII), `target_minor u16`, `target_major u16`, target date (u8-length ASCII).

## GetSerial body

`serial_len u32`, then `serial_len / 4` × `u32` groups formatted as `XXXXXXXX-`.

## FwSignature body

`signature u32`, filename (u8-length ASCII), id string (u8-length ASCII).
