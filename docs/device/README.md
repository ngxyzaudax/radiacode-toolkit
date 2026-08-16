# Device

The Device tab is the connection screen. It discovers RadiaCode detectors over USB and Bluetooth, lets you pick one to connect, and shows a compact summary if you return while already linked.

## Workflow

1. On launch the app scans automatically for nearby detectors.
2. Pick a device from the list or use **Reconnect** on the last-used device card.
3. After a successful connect the app switches to the **Monitor** tab automatically.
4. Return here anytime to disconnect or switch detectors.

## Features

- Centered connect layout with stable header and inline scan control
- **Last used** card with one-click reconnect when the remembered device is present
- **Available devices** list — compact clickable rows, USB first, then Bluetooth sorted by signal strength
- Transport icons, address, and RSSI for wireless links
- Connecting state with spinner and address confirmation
- Connected summary: model, serial, live battery / temperature / link chips, firmware line, **Disconnect**
- Remember last device and auto-connect on launch (Settings → Application → Connection)

## Related

- [Monitor](../monitor/README.md)
- [Settings](../settings/README.md)
- [Docs index](../README.md)
