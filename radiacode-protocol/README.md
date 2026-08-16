# radiacode-protocol

RadiaCode wire protocol — request framing, response assembly, opcode dispatch, and typed payload decoders.

Transport-agnostic: implements the `Transport` trait boundary but not USB or Bluetooth I/O.

## Contents

| Module | Role |
| --- | --- |
| `protocol` | Frame builder, sequence counter, response matcher |
| `command` | Opcode constants and SFR access helpers |
| `data_buf` | Monitor stream records — real-time rates, rare status, events |
| `spectrum` | Histogram decode and energy calibration helpers |
| `catalog` | SFR / configuration INI parsing |
| `rate_units` | Dose and count display unit conversions |
| `transport` | Async `Transport` trait |

## Reference

Full wire-format documentation: [docs/PROTOCOL.md](../docs/PROTOCOL.md).

## Usage

```toml
radiacode-protocol = { path = "../radiacode-protocol" }
```

```rust
use radiacode_protocol::{build_request, Command, ResponseAssembler, Transport};
```

## Examples

```bash
cargo run -p radiacode-protocol --example databuf_replay
```

## Tests

```bash
cargo test -p radiacode-protocol
```

## License

AGPL-3.0-only — see [LICENSE](../LICENSE).
