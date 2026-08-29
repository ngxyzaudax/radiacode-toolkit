# Transports

Both transports send the full framed request and reassemble the inner response via `ResponseAssembler`.

## USB

| Constant | Value |
| --- | --- |
| VID | `0x0483` |
| PID | `0xF123` |
| Interface | 0 |
| EP OUT | `0x01` |
| EP IN | `0x81` |
| Read buffer | 256 bytes |
| Timeout | 3000 ms |
| Empty read retries | 3 |

Flow: drain EP IN, single bulk write, read bulk IN until assembler completes, verify echoed header.

Linux udev rule: [`radiacode.rules`](../../radiacode.rules) (vendor `0483`, product `f123`).

## Bluetooth LE

| Role | UUID |
| --- | --- |
| Service | `e63215e5-7003-49d8-96b0-b024798fb901` |
| Write | `e63215e6-7003-49d8-96b0-b024798fb901` |
| Notify | `e63215e7-7003-49d8-96b0-b024798fb901` |

| Constant | Value |
| --- | --- |
| Write chunk size | 18 bytes (`WriteWithoutResponse`) |
| Response timeout | 30 s |
| Link settle after connect | 250 ms sleep + 500 ms drain |
| Pre-command drain | 400 ms peek + quiet window |

Flow: drain stale notifications, write request in 18-byte chunks, subscribe to notify characteristic, reassemble via `ResponseAssembler`, discard unrelated complete frames until echoed header matches.

No explicit ATT MTU negotiation; chunk size 18 implies ~20-byte ATT payload.
