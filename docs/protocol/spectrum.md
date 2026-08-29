# Spectrum compression

Header (all formats):

```text
u32 duration_sec
f32 a0
f32 a1
f32 a2
```

Format version comes from configuration INI `SpecFormatVersion=` (default 0).

## Format 0

Raw `u32` LE counts until buffer end.

## Format 1

Packed runs — each word is `u16` LE:

- Upper 12 bits: repeat count
- Lower 4 bits: vlen code (0..=5)

| vlen | Semantics |
| --- | --- |
| 0 | value = 0 |
| 1 | absolute `u8` |
| 2 | `last + i8` delta |
| 3 | `last + i16` delta |
| 4 | `last + ((i8<<16)\|(u8<<8)\|u8)` from bytes `a, b, c` |
| 5 | `last + i32` delta |

## Energy calibration

Coefficients `[a0, a1, a2]` map channel `c` to energy in keV:

```text
E = a0 + a1 * c + a2 * c²
```

Use `channel_to_energy(channel, a0, a1, a2)` from `radiacode-protocol`.

Write calibration via `WrVirtString(EnergyCalib)` with payload length and three `f32` values.

Reset live spectrum via `WrVirtString(Spectrum, len=0)`.
