# Messaging: nodes, send, listen, reply, info, support

## `nodes`

Lists all nodes currently known to the connected device.

```bash
meshtastic-cli nodes
```

Output columns:

| Column      | Description                                  |
|-------------|----------------------------------------------|
| ID          | Unique node identifier (hex)                 |
| Name        | Human-readable node name from device config  |
| Battery     | Battery level percentage (if reported)       |
| SNR         | Signal-to-noise ratio of the last packet     |
| Hops        | Number of hops from the local node           |
| Last Heard  | Timestamp of the most recent packet received |

Use `--fields` to select which columns to display. Separate field names with commas.

```bash
# Show only ID, name, and SNR
meshtastic-cli nodes --fields id,name,snr

# Show extended fields including hardware model, role, and position
meshtastic-cli nodes --fields id,name,hw_model,role,position
```

Available fields:

| Field       | Description                                     | Default |
|-------------|-------------------------------------------------|---------|
| `id`        | Node identifier (hex)                           | Yes     |
| `name`      | Node long name                                  | Yes     |
| `battery`   | Battery level percentage                        | Yes     |
| `snr`       | Signal-to-noise ratio                           | Yes     |
| `hops`      | Number of hops from local node                  | Yes     |
| `last_heard`| Timestamp of last received packet               | Yes     |
| `hw_model`  | Hardware model name                             | No      |
| `role`      | Device role (CLIENT, ROUTER, etc.)              | No      |
| `position`  | Last known GPS coordinates                      | No      |

---

## `send`

Sends a text message to the mesh network. By default the message is broadcast to all nodes.

```bash
# Broadcast a message to all nodes
meshtastic-cli send "hello mesh"

# Send to a specific node by hex ID
meshtastic-cli send "hello node" --dest 04e1c43b

# Send to a node by name (searches known nodes, case-insensitive)
meshtastic-cli send "hello!" --to Pedro

# Send on a specific channel (0-7)
meshtastic-cli send "hello channel" --channel 1

# Combine destination and channel
meshtastic-cli send "direct message" --dest 04e1c43b --channel 2

# Wait for delivery confirmation (ACK) before returning
meshtastic-cli send "confirmed message" --dest 04e1c43b --ack

# Wait for ACK with custom timeout
meshtastic-cli send "confirmed message" --to Pedro --ack --timeout 60

# Send as a private message (PRIVATE_APP port instead of text port)
meshtastic-cli send "private payload" --dest 04e1c43b --private
```

> **Shell note:** The `!` prefix is optional. If you include it, quote or escape it to prevent shell history expansion: `--dest '!04e1c43b'` or `--dest \!04e1c43b`.

| Option      | Description                                            |
|-------------|--------------------------------------------------------|
| `<MESSAGE>` | The text message to send (required, positional)        |
| `--dest`    | Destination node ID in hex (e.g. `04e1c43b`). The `!` prefix is optional. Cannot be combined with `--to`. |
| `--to`      | Destination node name (e.g. `Pedro`). Searches known nodes by name (case-insensitive). If multiple nodes match, shows the list and asks you to use `--dest` instead. Cannot be combined with `--dest`. |
| `--channel` | Channel index 0-7 (default: 0)                        |
| `--ack`     | Wait for delivery ACK before returning. Requires `--dest` or `--to` (cannot ACK a broadcast). |
| `--timeout` | Seconds to wait for ACK when `--ack` is set (default: 30). |
| `--private` | Send on PRIVATE_APP port (port 256) instead of the standard text message port. |

---

## `listen`

Streams all incoming packets from the mesh network in real time. Runs continuously until interrupted with Ctrl+C.

```bash
meshtastic-cli listen

# Write all received packets as JSON Lines to a log file
meshtastic-cli listen --log packets.jsonl

# Continue displaying packets in the terminal while also writing to a log file
meshtastic-cli listen --log /var/log/meshtastic/packets.jsonl
```

Decodes and displays the following packet types:

| Packet Type  | Display                                          |
|--------------|--------------------------------------------------|
| Text message | Full message text                                |
| Position     | Latitude, longitude, altitude, satellite count   |
| Telemetry    | Battery, voltage, channel utilization, env data  |
| Node info    | Long name, short name                            |
| Routing      | ACK/NAK status, route requests/replies           |
| Other        | Port type and payload size                       |

| Option  | Description                                                                     |
|---------|---------------------------------------------------------------------------------|
| `--log` | File path to write received packets as JSON Lines (one JSON object per line). The terminal display continues in parallel. Omit to disable file logging. |

Example output:

```
-> Listening for packets... Press Ctrl+C to stop.

[15:30:00] !04e1c43b (Pedro) -> broadcast      | Text: Hello everyone!
[15:30:05] !a1b2c3d4 (Maria) -> !04e1c43b      | Position: 40.41680, -3.70380, 650m, 8 sats
[15:30:10] !04e1c43b (Pedro) -> broadcast       | Telemetry: battery 85%, 3.90V, ch_util 12.3%
[15:30:15] !a1b2c3d4 (Maria) -> !04e1c43b      | Routing: ACK
```

---

## `reply`

Auto-reply mode. Listens for incoming text messages and automatically replies to each sender with signal information (SNR, RSSI, hops). Useful for range testing and network debugging. Runs continuously until interrupted with Ctrl+C.

```bash
meshtastic-cli reply
```

Example output:

```
-> Reply mode active. Listening for messages. Press Ctrl+C to stop.

[15:30:00] Message from Pedro (!04e1c43b): "hello"
-> Replied: "Heard you! SNR: 8.5 dB, RSSI: -85 dBm, Hops: 2"
```

---

## `info`

Displays detailed information about the local node and connected device.

```bash
meshtastic-cli info
```

Example output:

```
Node
  ID:              !04e1c43b
  Name:            Pedro
  Short name:      PD
  Hardware:        HELTEC V3
  Role:            CLIENT

Firmware
  Version:         2.5.6.abc1234
  Reboots:         12

Capabilities
  Features:        WiFi, Bluetooth, PKC

Device Metrics
  Battery:         85%
  Voltage:         3.90V
  Channel util.:   12.3%
  Uptime:          2d 5h 30m

Channels
  Ch 0:            Default (Primary, AES-256)
  Ch 1:            Team (Secondary, AES-256)

  Nodes in mesh:   8
```

---

## `support`

Displays a diagnostic summary of the connected device and CLI. Useful for troubleshooting and for sharing device context in bug reports.

```bash
meshtastic-cli support
```

Example output:

```
meshtastic-cli v0.3.0

Device
  Node ID:        !04e1c43b
  Firmware:       2.5.6.abc1234
  Hardware:       HELTEC_V3
  Role:           CLIENT
  Region:         EU868
  Modem preset:   LongFast
  Capabilities:   WiFi, Bluetooth, PKC

Channels
  [0] Default (Primary)
  [1] Team (Secondary)

Known nodes: 8
```
