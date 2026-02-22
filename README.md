# meshtastic-cli

> A Rust CLI tool for interacting with Meshtastic mesh networking devices over TCP, serial, or BLE connections.

## Purpose & Context

**What it does**: `meshtastic-cli` provides a command-line interface to Meshtastic devices, allowing you to list nodes, send messages, monitor incoming packets, query device info, ping specific nodes, manage channels, control GPIO pins, and more — all from a terminal.

**Why it exists**: The Meshtastic ecosystem lacks a robust, composable CLI tool built in Rust. This project aims to fill that gap as an open-source contribution, leveraging the official `meshtastic` Rust crate to interact with real hardware and local simulators alike.

**Who it's for**: Developers and operators working with Meshtastic mesh networks who want scriptable, terminal-native access to device data without a GUI.

**Key Design Decisions**:
- Strategy pattern for commands: each command is an independent module implementing a shared trait, making it trivial to add new commands without touching existing ones.
- SOLID principles throughout: single responsibility per module, open/closed for command extension, dependency inversion via connection abstraction.
- Thin `main.rs`: only parses CLI arguments and dispatches to the appropriate command — no business logic lives there.
- Async-first: all I/O uses Tokio, matching the async model of the underlying `meshtastic` crate.
- Optional BLE support: compiled in via `--features ble` to avoid requiring Bluetooth dependencies in environments that do not need them.

---

## Features (v0.3.0)

- TCP connectivity to local simulators or remote devices
- Serial connectivity to physical Meshtastic hardware
- BLE connectivity to nearby devices (requires `--features ble` build)
- `--no-nodes` flag: skip node collection on startup for faster command execution
- `--json` flag: output results as JSON for all data-returning commands (scriptable pipelines)
- `nodes` command: list all mesh nodes with ID, name, battery level, SNR, hop count, and last-heard timestamp; supports `--fields` column filtering
- `send` command: send text messages to the mesh (broadcast, by node ID, by node name, or on a specific channel); supports `--ack` for delivery confirmation and `--private` for private-port messaging
- `listen` command: stream and decode incoming packets in real time (text, position, telemetry, routing, node info); supports `--log` to write packets as JSON Lines to a file
- `info` command: display local node details, firmware, capabilities, channels, device metrics, and position
- `ping` command: ping a specific node by ID or name, measure round-trip time, with configurable timeout
- `config get` command: display all or individual device/module configuration sections
- `config set` command: modify any configuration field with automatic device reboot
- `config begin-edit` / `config commit-edit` commands: batch config change signaling
- `config set-modem-preset` command: set modem preset directly by name
- `config ch-add-url` command: add channels from a meshtastic:// URL without replacing existing channels
- `traceroute` command: trace route to a node showing each hop with SNR values
- `channel` command: add, delete, list, set properties, and generate QR codes for channels; supports `--all` for per-channel QR output
- `config export` command: export full device configuration (config, module config, channels) to YAML
- `config import` command: import and apply configuration from a YAML file
- `device reboot` command: reboot local or remote device with configurable delay
- `device reboot-ota` command: reboot into OTA firmware update mode (ESP32 devices)
- `device enter-dfu` command: enter DFU mode (NRF52 devices)
- `device shutdown` command: shut down local or remote device with configurable delay
- `device set-time` command: set the device clock from a Unix timestamp or system time
- `device set-canned-message` command: configure canned message slots separated by `|`
- `device get-canned-message` command: display currently configured canned messages
- `device set-ringtone` command: set the notification ringtone in RTTTL format
- `device get-ringtone` command: display the currently stored ringtone
- `device factory-reset` command: restore factory defaults (preserves BLE bonds)
- `device factory-reset-device` command: full factory reset including BLE bond wipe
- `device reset-nodedb` command: clear the node database
- `node set-owner` command: set device long name and short name
- `node set-favorite` command: mark a node as favorite
- `node remove-favorite` command: remove a node from favorites
- `node set-ignored` command: mark a node as ignored
- `node remove-ignored` command: remove a node from the ignored list
- `node remove` command: remove a specific node from the local NodeDB
- `node set-unmessageable` command: mark the local node as unmessageable (prevents others from messaging it)
- `position get` command: display current GPS position
- `position set` command: set a fixed GPS position (latitude, longitude, altitude, optional named broadcast flags)
- `position remove` command: clear the fixed GPS position and return to GPS-based positioning
- `request telemetry` command: request telemetry from a remote node with `--type` selection (device, environment, air-quality, power, local-stats, health, host)
- `request position` command: request position from a remote node
- `request metadata` command: request device metadata from a remote node
- `config set-ham` command: configure licensed Ham radio mode with callsign
- `config set-url` command: apply channels and LoRa config from a meshtastic:// URL (replaces existing channels)
- `reply` command: auto-reply mode — listens for incoming text messages and responds with signal info (SNR, RSSI, hops)
- `support` command: display diagnostic info (CLI version, firmware, hardware, channels, known nodes, region, modem preset)
- `gpio write/read/watch` commands: remote GPIO pin operations on mesh nodes
- `completions` command: generate shell completion scripts for bash, zsh, fish, PowerShell, and Elvish
- `config-file` command: manage persistent CLI configuration at `~/.config/meshtastic-cli/config.toml` (show, set, unset, path)
- `waypoint` command: waypoint management — send, delete, and list waypoints on the mesh
- `watch` command: live-updating node table with periodic refresh (replaces the static `nodes` output)
- `mqtt bridge` command: bidirectional MQTT bridge — republish mesh packets to an MQTT broker and inject MQTT messages into the mesh
- `shell` command: interactive REPL with command history and tab completion for exploratory sessions
- Colored terminal output for readability
- Docker simulator support for local development without hardware

---

## Installation

### Pre-built binaries (recommended)

Download the latest binary for your platform from [GitHub Releases](https://github.com/matutetandil/meshtastic-cli/releases):

| Platform | Binary |
|---|---|
| Linux x86_64 | `meshtastic-cli-linux-x86_64` |
| Linux ARM64 | `meshtastic-cli-linux-aarch64` |
| macOS Intel | `meshtastic-cli-macos-x86_64` |
| macOS Apple Silicon | `meshtastic-cli-macos-aarch64` |
| Windows x86_64 | `meshtastic-cli-windows-x86_64.exe` |

```bash
# Example: Linux x86_64
curl -L https://github.com/matutetandil/meshtastic-cli/releases/latest/download/meshtastic-cli-linux-x86_64 -o meshtastic-cli
chmod +x meshtastic-cli
sudo mv meshtastic-cli /usr/local/bin/

# Example: macOS Apple Silicon
curl -L https://github.com/matutetandil/meshtastic-cli/releases/latest/download/meshtastic-cli-macos-aarch64 -o meshtastic-cli
chmod +x meshtastic-cli
sudo mv meshtastic-cli /usr/local/bin/
```

### Install from crates.io

If you have the Rust toolchain installed:

```bash
cargo install meshtastic-cli
```

### Build from source

```bash
git clone https://github.com/matutetandil/meshtastic-cli.git
cd meshtastic-cli
cargo build --release
```

The compiled binary will be at `target/release/meshtastic-cli`.

### BLE support

All pre-built binaries include BLE support out of the box. No extra steps needed.

When building from source, add the `ble` feature flag:

```bash
cargo build --release --features ble
# or
cargo install meshtastic-cli --features ble
```

Linux requires BlueZ: `sudo apt install libbluetooth-dev`

---

## Quick Start

### Option A: Use the Docker simulator (no hardware required)

1. The repository includes a `config.yaml` for the simulator. Start it:

```bash
docker run -d --name meshtasticd \
  -v ./config.yaml:/etc/meshtasticd/config.yaml:ro \
  -p 4403:4403 \
  meshtastic/meshtasticd:latest meshtasticd -s
```

2. List nodes:

```bash
meshtastic-cli nodes
```

### Option B: Connect to physical hardware via serial

```bash
meshtastic-cli --serial /dev/ttyUSB0 nodes
```

---

## Usage

```
meshtastic-cli [OPTIONS] <COMMAND>

Options:
  --host <HOST>        TCP host to connect to [default: 127.0.0.1]
  --port <PORT>        TCP port to connect to [default: 4403]
  --serial <PATH>      Serial device path (e.g. /dev/ttyUSB0). Overrides TCP.
  --ble <NAME|MAC>     BLE device name or MAC address (requires --features ble build)
  --ble-scan           Scan for nearby BLE Meshtastic devices and list them
  --no-nodes           Skip node collection during connection (faster startup)
  --json               Output results as JSON instead of formatted text
  -h, --help           Print help
  -V, --version        Print version

Commands:
  nodes        List all nodes visible on the mesh
  send         Send a text message to the mesh network
  listen       Stream incoming packets in real time
  info         Show local node and device information
  reply        Auto-reply to incoming messages with signal info
  support      Display diagnostic info about the connected device and CLI
  config       Get, set, export, import, set-ham, set-url, begin-edit, commit-edit, set-modem-preset, ch-add-url
  node         Node management (set-owner, remove, set-favorite, remove-favorite, set-ignored, remove-ignored)
  device       Device management (reboot, reboot-ota, enter-dfu, shutdown, factory-reset, factory-reset-device, reset-nodedb, set-time, set-canned-message, get-canned-message, set-ringtone, get-ringtone)
  channel      Manage channels (add, delete, set, list, qr)
  position     GPS position (get, set, remove)
  request      Request data from remote nodes (telemetry, position, metadata)
  traceroute   Trace route to a node showing each hop
  ping         Ping a node and measure round-trip time
  gpio         Remote GPIO operations (write, read, watch)
  config-file  Manage persistent CLI configuration (~/.config/meshtastic-cli/config.toml)
  waypoint     Waypoint management (send, delete, list)
  watch        Live-updating node table with periodic refresh
  mqtt         MQTT bridge (bridge)
  shell        Interactive REPL with command history and tab completion
  completions  Generate shell completion scripts
```

### Connection examples

```bash
# TCP — default (connects to localhost:4403, ideal for Docker simulator)
meshtastic-cli nodes

# TCP — custom host and port
meshtastic-cli --host 192.168.1.100 --port 4403 nodes

# Serial — physical device
meshtastic-cli --serial /dev/ttyUSB0 nodes

# BLE — connect by device name (requires --features ble build)
meshtastic-cli --ble "Meshtastic_abcd" nodes

# BLE — connect by MAC address
meshtastic-cli --ble "AA:BB:CC:DD:EE:FF" nodes

# BLE — scan for nearby devices
meshtastic-cli --ble-scan

# Skip node collection for faster startup (useful for commands that don't need node info)
meshtastic-cli --no-nodes send "hello mesh"
```

---

## Commands

### `nodes`

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

### `send`

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

### `listen`

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

### `reply`

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

### `support`

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

### `info`

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

### `--json` global flag

Append `--json` to any data-returning command to receive output as a JSON object or array instead of the default formatted text. This is useful for shell scripting, log ingestion, or piping output into tools like `jq`.

```bash
# List nodes as JSON
meshtastic-cli --json nodes

# Get device config as JSON
meshtastic-cli --json config get lora

# Get local node info as JSON
meshtastic-cli --json info

# Get position as JSON
meshtastic-cli --json position get

# Request remote telemetry as JSON
meshtastic-cli --json request telemetry --dest 04e1c43b

# Pipe into jq for filtering
meshtastic-cli --json nodes | jq '[.[] | select(.battery < 20)]'
```

The flag is a global option and must be placed before the subcommand name. Commands that produce no structured output (e.g., `send`, `device reboot`) ignore the flag.

### `config`

Read, write, export, and import device and module configuration. Supports all 8 device config sections and 13 module config sections.

#### `config get`

Display current configuration. Optionally specify a section to show only that section.

```bash
# Show all configuration sections
meshtastic-cli config get

# Show a specific section
meshtastic-cli config get lora
meshtastic-cli config get mqtt
meshtastic-cli config get device
```

Available sections:

| Device Config | Module Config |
|--------------|---------------|
| `device` | `mqtt` |
| `position` | `serial` |
| `power` | `external-notification` |
| `network` | `store-forward` |
| `display` | `range-test` |
| `lora` | `telemetry` |
| `bluetooth` | `canned-message` |
| `security` | `audio` |
| | `remote-hardware` |
| | `neighbor-info` |
| | `ambient-lighting` |
| | `detection-sensor` |
| | `paxcounter` |

Example output:

```
LoRa
  region:                                  Us
  modem_preset:                            LongFast
  use_preset:                              true
  hop_limit:                               3
  tx_enabled:                              true
  tx_power:                                30
  ...
```

#### `config set`

Set a configuration value. The key uses `section.field` format. The device will reboot after applying changes.

```bash
# Set LoRa region
meshtastic-cli config set lora.region Eu868

# Change device role
meshtastic-cli config set device.role Router

# Set hop limit
meshtastic-cli config set lora.hop_limit 5

# Enable MQTT
meshtastic-cli config set mqtt.enabled true

# Set WiFi credentials
meshtastic-cli config set network.wifi_ssid "MyNetwork"
meshtastic-cli config set network.wifi_psk "MyPassword"
```

For enum fields, use the human-readable name (case-insensitive). Run `config get <section>` to see current values and available field names.

Example output:

```
-> Setting lora.region = Eu868
! Device will reboot to apply changes.
ok Configuration updated.
```

#### `config begin-edit`

Signal the device to begin collecting a batch of configuration changes. Use this before a sequence of `config set` calls to apply them all in a single transaction rather than rebooting after each change.

```bash
meshtastic-cli config begin-edit
meshtastic-cli config set lora.region Eu868
meshtastic-cli config set lora.hop_limit 5
meshtastic-cli config set device.role Router
meshtastic-cli config commit-edit
```

#### `config commit-edit`

Signal the device to commit and apply all configuration changes queued since the last `config begin-edit`. The device will reboot once to apply all pending changes.

```bash
meshtastic-cli config commit-edit
```

#### `config set-modem-preset`

Set the LoRa modem preset directly by name, without having to go through `config set lora.modem_preset`. Valid preset names are case-insensitive.

```bash
meshtastic-cli config set-modem-preset LongFast
meshtastic-cli config set-modem-preset ShortTurbo
meshtastic-cli config set-modem-preset MediumSlow
```

Available presets:

| Preset | Description |
|---|---|
| `LongFast` | Long range, faster throughput (default) |
| `LongSlow` | Long range, slower throughput |
| `VeryLongSlow` | Maximum range, very slow |
| `MediumSlow` | Medium range, slower |
| `MediumFast` | Medium range, faster |
| `ShortSlow` | Short range, slower |
| `ShortFast` | Short range, fastest throughput |
| `LongModerate` | Long range, moderate throughput |
| `ShortTurbo` | Short range, maximum throughput |

#### `config ch-add-url`

Add channels from a meshtastic:// URL without replacing existing channels. This differs from `config set-url`, which replaces all current channels with those from the URL.

```bash
meshtastic-cli config ch-add-url "https://meshtastic.org/e/#ENCODED..."
```

| Option | Description |
|---|---|
| `<URL>` | Meshtastic configuration URL (required) |

### `node`

Node management commands.

#### `node set-owner`

Set the device owner name (long name and short name). The short name is auto-generated from the long name if omitted.

```bash
# Set long name (short name auto-generated as "PD")
meshtastic-cli node set-owner "Pedro"

# Set both long and short name
meshtastic-cli node set-owner "Pedro's Node" --short PN

# Multi-word names generate initials (e.g. "My Cool Node" -> "MCN")
meshtastic-cli node set-owner "My Cool Node"
```

| Option | Description |
|---|---|
| `<NAME>` | Long name for the device, up to 40 characters (required) |
| `--short` | Short name, up to 5 characters. Auto-generated if omitted |

#### `node remove`

Remove a specific node from the local NodeDB. The node can be specified by hex ID or by name.

```bash
# Remove by node ID
meshtastic-cli node remove --dest 04e1c43b

# Remove by name
meshtastic-cli node remove --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex to remove (required unless `--to` is used) |
| `--to` | Node name to remove (required unless `--dest` is used) |

#### `node set-favorite`

Mark a node as a favorite. Favorites are stored on the device and can be used for filtering in compatible clients.

```bash
# Mark by node ID
meshtastic-cli node set-favorite --dest 04e1c43b

# Mark by name
meshtastic-cli node set-favorite --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

#### `node remove-favorite`

Remove a node from the favorites list.

```bash
meshtastic-cli node remove-favorite --dest 04e1c43b
meshtastic-cli node remove-favorite --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

#### `node set-ignored`

Mark a node as ignored. Ignored nodes are filtered out of mesh activity on the local device.

```bash
meshtastic-cli node set-ignored --dest 04e1c43b
meshtastic-cli node set-ignored --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

#### `node remove-ignored`

Remove a node from the ignored list.

```bash
meshtastic-cli node remove-ignored --dest 04e1c43b
meshtastic-cli node remove-ignored --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

#### `node set-unmessageable`

Mark the local node as unmessageable (prevents others from sending direct messages to it) or restore it as messageable.

```bash
# Mark as unmessageable (default)
meshtastic-cli node set-unmessageable

# Explicitly mark as unmessageable
meshtastic-cli node set-unmessageable true

# Restore as messageable
meshtastic-cli node set-unmessageable false
```

| Option | Description |
|---|---|
| `[VALUE]` | `true` to mark as unmessageable, `false` to mark as messageable (default: `true`) |

### `device`

Device management commands: reboot, reboot-ota, enter-dfu, shutdown, factory reset variants, reset-nodedb, set-time, canned messages, and ringtone. Reboot and shutdown support targeting the local device (default) or a remote node.

#### `device reboot`

Reboot the connected device or a remote node.

```bash
# Reboot local device (5 second delay)
meshtastic-cli device reboot

# Reboot with custom delay
meshtastic-cli device reboot --delay 10

# Reboot a remote node by ID
meshtastic-cli device reboot --dest 04e1c43b

# Reboot a remote node by name
meshtastic-cli device reboot --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex. Omit to target local device |
| `--to` | Target node name. Omit to target local device |
| `--delay` | Seconds before rebooting (default: 5) |

#### `device reboot-ota`

Reboot the device into OTA (Over-The-Air) firmware update mode. This is specific to ESP32-based Meshtastic hardware. Supports targeting the local device or a remote node.

```bash
# Reboot local device into OTA mode
meshtastic-cli device reboot-ota

# Reboot remote node into OTA mode
meshtastic-cli device reboot-ota --dest 04e1c43b
meshtastic-cli device reboot-ota --to Pedro

# Custom delay
meshtastic-cli device reboot-ota --delay 10
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex. Omit to target local device |
| `--to` | Target node name. Omit to target local device |
| `--delay` | Seconds before rebooting into OTA mode (default: 5) |

#### `device enter-dfu`

Enter Device Firmware Upgrade (DFU) mode. This is specific to NRF52-based Meshtastic hardware (e.g., RAK devices). The device will appear as a USB mass storage device after entering DFU mode, allowing firmware file drops.

```bash
meshtastic-cli device enter-dfu
```

#### `device shutdown`

Shut down the connected device or a remote node.

```bash
# Shutdown local device
meshtastic-cli device shutdown

# Shutdown with custom delay
meshtastic-cli device shutdown --delay 10

# Shutdown a remote node
meshtastic-cli device shutdown --dest 04e1c43b
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex. Omit to target local device |
| `--to` | Target node name. Omit to target local device |
| `--delay` | Seconds before shutting down (default: 5) |

#### `device factory-reset`

Restore the device to factory defaults. This erases all configuration and stored data but **preserves BLE bonds**.

```bash
meshtastic-cli device factory-reset
```

#### `device factory-reset-device`

Perform a full factory reset that also **wipes all BLE bonds**. Use this when you want to completely reset the device as if it were brand new, including removing all previously paired Bluetooth devices.

```bash
meshtastic-cli device factory-reset-device
```

#### `device reset-nodedb`

Clear the device's entire node database. This removes all known nodes from the local NodeDB.

```bash
meshtastic-cli device reset-nodedb
```

#### `device set-time`

Set the device clock. Uses the current system time if no timestamp is provided.

```bash
# Set time from system clock
meshtastic-cli device set-time

# Set time from a specific Unix timestamp
meshtastic-cli device set-time 1708444800
```

| Option | Description |
|---|---|
| `[TIMESTAMP]` | Unix timestamp in seconds. Uses system time if omitted |

#### `device set-canned-message`

Set the canned messages stored on the device. Messages are separated by `|` and can be selected quickly from a compatible Meshtastic client.

```bash
meshtastic-cli device set-canned-message "Yes|No|Help|On my way|Call me"
```

| Option | Description |
|---|---|
| `<MESSAGES>` | Pipe-separated list of canned messages (required) |

#### `device get-canned-message`

Display the canned messages currently configured on the device. Requests the canned message module config from the device and waits for the response.

```bash
meshtastic-cli device get-canned-message

# Custom timeout
meshtastic-cli device get-canned-message --timeout 60
```

| Option | Description |
|---|---|
| `--timeout` | Seconds to wait for the device response (default: 30) |

Example output:

```
Canned messages:
  1: Yes
  2: No
  3: Help
  4: On my way
  5: Call me
```

#### `device set-ringtone`

Set the notification ringtone on the device. The ringtone is provided in RTTTL (Ring Tone Text Transfer Language) format.

```bash
meshtastic-cli device set-ringtone "scale:d=4,o=5,b=120:c,e,g,c6"
```

| Option | Description |
|---|---|
| `<RINGTONE>` | Ringtone string in RTTTL format (required) |

#### `device get-ringtone`

Display the notification ringtone currently stored on the device.

```bash
meshtastic-cli device get-ringtone

# Custom timeout
meshtastic-cli device get-ringtone --timeout 60
```

| Option | Description |
|---|---|
| `--timeout` | Seconds to wait for the device response (default: 30) |

Example output:

```
Ringtone: scale:d=4,o=5,b=120:c,e,g,c6
```

### `channel`

Manage device channels: list, add, delete, modify properties, and generate a QR code for sharing.

#### `channel list`

List all configured channels with their role, encryption, and uplink/downlink status.

```bash
meshtastic-cli channel list
```

Example output:

```
Channels
  [0]    Default        Primary      Default key  uplink: false downlink: false
  [1]    Team           Secondary    AES-256      uplink: false downlink: false
```

#### `channel add`

Add a new secondary channel. The channel is placed in the first available slot (indices 1-7).

```bash
# Add with default encryption key
meshtastic-cli channel add "Team"

# Add with a random AES-256 key
meshtastic-cli channel add "Secure" --psk random

# Add with no encryption
meshtastic-cli channel add "Open" --psk none

# Add with a specific AES-128 key (32 hex characters)
meshtastic-cli channel add "Custom" --psk d4f1bb3a2029075960bcffabcf4e6901
```

| Option | Description |
|---|---|
| `<NAME>` | Channel name, up to 11 characters (required) |
| `--psk` | Pre-shared key: `none`, `default`, `random`, or hex-encoded key (default: `default`) |

#### `channel del`

Delete a channel by index. Cannot delete the primary channel (index 0).

```bash
meshtastic-cli channel del 1
```

#### `channel set`

Set a property on a specific channel.

```bash
# Rename a channel
meshtastic-cli channel set 1 name "NewName"

# Change encryption key
meshtastic-cli channel set 1 psk random

# Enable MQTT uplink
meshtastic-cli channel set 1 uplink_enabled true

# Enable MQTT downlink
meshtastic-cli channel set 1 downlink_enabled true

# Set position precision
meshtastic-cli channel set 0 position_precision 14
```

| Field | Description |
|---|---|
| `name` | Channel name (up to 11 characters) |
| `psk` | Pre-shared key (`none`, `default`, `random`, or hex) |
| `uplink_enabled` | Forward mesh messages to MQTT |
| `downlink_enabled` | Forward MQTT messages to mesh |
| `position_precision` | Bits of precision for position data |

#### `channel qr`

Generate a QR code and shareable meshtastic:// URL for the current channel configuration. By default the QR code is printed directly to the terminal using Unicode block characters. Use `--output` to save as a PNG or SVG image file. Use `--all` to generate a separate QR code for each active channel individually.

```bash
# Print combined QR code to terminal (all active channels)
meshtastic-cli channel qr

# Export combined QR as PNG image (512x512 minimum)
meshtastic-cli channel qr --output channels.png

# Export combined QR as SVG image
meshtastic-cli channel qr --output channels.svg

# Print individual QR code per active channel to terminal
meshtastic-cli channel qr --all
```

| Option | Description |
|---|---|
| `--output` | File path for image export. Supports `.png` and `.svg` formats. Prints to terminal if omitted. Cannot be combined with `--all`. |
| `--all` | Generate one QR code per active channel, printed to terminal. Cannot be combined with `--output`. |

Example output (terminal):

```
[block character QR code rendered in terminal]

URL: https://meshtastic.org/e/#ENCODED...
```

Example output (file export):

```
ok QR code saved to channels.png

URL: https://meshtastic.org/e/#ENCODED...
```

Example output (`--all`, two active channels):

```
Channel 0: Default
[block character QR code]
URL: https://meshtastic.org/e/#ENCODED_CH0...

Channel 1: Team
[block character QR code]
URL: https://meshtastic.org/e/#ENCODED_CH1...
```

#### `config export`

Exports the full device configuration (device config, module config, and channels) as YAML. Useful for backups, sharing configurations, or migrating between devices.

```bash
# Print config to stdout
meshtastic-cli config export

# Save to a file
meshtastic-cli config export --file backup.yaml
```

| Option | Description |
|---|---|
| `--file` | Output file path. If omitted, prints YAML to stdout |

Example output (truncated):

```yaml
bluetooth:
  enabled: true
  fixed_pin: 123456
  mode: 1
device:
  role: 0
  node_info_broadcast_secs: 900
  ...
lora:
  region: 1
  modem_preset: 3
  hop_limit: 3
  ...
mqtt:
  enabled: false
  address: mqtt.meshtastic.org
  ...
channels:
  - index: 0
    role: PRIMARY
    name: ''
    psk: '01'
    uplink_enabled: false
    downlink_enabled: false
    position_precision: 0
  - index: 1
    role: SECONDARY
    name: Team
    psk: d4f1bb3a2029075960bcffabcf4e6901...
    ...
```

#### `config import`

Imports and applies configuration from a YAML file. The file format matches the output of `config export`. Sections not present in the file are left unchanged. The device will reboot after applying config changes.

```bash
# Import from a file
meshtastic-cli config import backup.yaml
```

| Option | Description |
|---|---|
| `<FILE>` | Path to the YAML configuration file (required) |

Example output:

```
-> Importing configuration from backup.yaml...
ok Imported 8 config sections, 13 module sections, 2 channels.
! Device will reboot to apply configuration changes.
```

#### `config set-ham`

Configure the device for licensed Ham radio operation. Sets the callsign as the long name, enables long-range LoRa settings, and disables encryption as required by Ham regulations. Optionally set TX power and frequency.

```bash
# Set Ham mode with callsign
meshtastic-cli config set-ham KD2ABC

# Set Ham mode with custom TX power and frequency
meshtastic-cli config set-ham KD2ABC --tx-power 17 --frequency 906.875
```

| Option | Description |
|---|---|
| `<CALLSIGN>` | Ham radio callsign to set as device name (required) |
| `--tx-power` | Transmit power in dBm (optional) |
| `--frequency` | Frequency in MHz (optional) |

#### `config set-url`

Apply channels and LoRa configuration from a meshtastic:// URL. These URLs are typically generated by the Meshtastic app or web client for sharing device configurations. **This replaces all existing channels** with those defined in the URL. To add channels without replacing existing ones, use `config ch-add-url`.

```bash
meshtastic-cli config set-url "https://meshtastic.org/e/#ENCODED..."
```

| Option | Description |
|---|---|
| `<URL>` | Meshtastic configuration URL (required) |

### `traceroute`

Traces the route to a destination node, showing each hop along the path with SNR (signal-to-noise ratio) values.

```bash
# Traceroute by node ID
meshtastic-cli traceroute --dest 04e1c43b

# Traceroute by name
meshtastic-cli traceroute --to Pedro

# Custom timeout (default: 60s)
meshtastic-cli traceroute --dest 04e1c43b --timeout 120
```

| Option | Description |
|---|---|
| `--dest` | Destination node ID in hex, `!` prefix optional (required unless `--to` is used) |
| `--to` | Destination node name (required unless `--dest` is used) |
| `--timeout` | Seconds to wait for response (default: 60) |

Example output:

```
-> Tracing route to Pedro (!04e1c43b)...

  1 !a1b2c3d4 (Local)
  2 !e5f6a7b8 (Relay-1)     SNR: 6.0 dB
  3 !04e1c43b (Pedro)        SNR: 8.5 dB

ok Route to Pedro (!04e1c43b) completed in 4.2s (2 hops)
```

If a return path differs from the forward path, both are shown separately.

### `ping`

Sends a ping to a specific node and measures the round-trip time by waiting for an ACK.

```bash
# Ping by node ID
meshtastic-cli ping --dest 04e1c43b

# Ping by name
meshtastic-cli ping --to Pedro

# Custom timeout (default: 30s)
meshtastic-cli ping --dest 04e1c43b --timeout 60
```

| Option      | Description                                            |
|-------------|--------------------------------------------------------|
| `--dest`    | Destination node ID in hex, `!` prefix optional (required unless `--to` is used) |
| `--to`      | Destination node name (required unless `--dest` is used) |
| `--timeout` | Seconds to wait for ACK (default: 30)                  |

Example output:

```
-> Pinging !04e1c43b (Pedro) (packet id: a1b2c3d4)...
ok ACK from !04e1c43b (Pedro) in 2.3s
```

If the node doesn't respond:

```
-> Pinging !04e1c43b (Pedro) (packet id: a1b2c3d4)...
x Timeout after 30s -- no ACK from !04e1c43b (Pedro)
```

### `position`

GPS position commands: get, set, and remove.

#### `position get`

Display the current GPS position of the local node.

```bash
meshtastic-cli position get
```

#### `position set`

Set a fixed GPS position on the device. Requires latitude and longitude; altitude and broadcast flags are optional. Once a fixed position is set, the device broadcasts this position instead of using live GPS data.

```bash
# Set position with latitude and longitude
meshtastic-cli position set 40.4168 -3.7038

# Set position with altitude (in meters)
meshtastic-cli position set 40.4168 -3.7038 650

# Set position with named broadcast flags
meshtastic-cli position set 40.4168 -3.7038 650 --flags "ALTITUDE,TIMESTAMP,SPEED"

# Set position with numeric bitmask (equivalent to above: 1 + 128 + 512 = 641)
meshtastic-cli position set 40.4168 -3.7038 650 --flags 641

# Set position with hex bitmask
meshtastic-cli position set 40.4168 -3.7038 650 --flags 0x281
```

| Option | Description |
|---|---|
| `<LATITUDE>` | Latitude in decimal degrees (required) |
| `<LONGITUDE>` | Longitude in decimal degrees (required) |
| `<ALTITUDE>` | Altitude in meters (optional) |
| `--flags` | Position broadcast field flags (optional). Accepts comma-separated names (`ALTITUDE`, `ALTITUDE_MSL`, `GEOIDAL_SEPARATION`, `DOP`, `HVDOP`, `SATINVIEW`, `SEQ_NO`, `TIMESTAMP`, `HEADING`, `SPEED`) or a numeric bitmask (decimal or `0x` hex). |

#### `position remove`

Remove the fixed GPS position from the device. After removal, the device will return to using live GPS data if a GPS module is available.

```bash
meshtastic-cli position remove
```

### `request`

Request data from remote nodes.

#### `request telemetry`

Request telemetry data from a remote node. Use `--type` to select a specific telemetry variant (default: device).

```bash
# Request device telemetry (battery, voltage, channel utilization)
meshtastic-cli request telemetry --dest 04e1c43b

# Request environment telemetry (temperature, humidity, pressure)
meshtastic-cli request telemetry --to Pedro --type environment

# Request air quality metrics (PM1.0, PM2.5, PM10.0, CO2, VOC)
meshtastic-cli request telemetry --dest 04e1c43b --type air-quality

# Request power metrics (voltage/current per channel)
meshtastic-cli request telemetry --dest 04e1c43b --type power

# Request local stats (uptime, packets tx/rx, air utilization)
meshtastic-cli request telemetry --dest 04e1c43b --type local-stats

# Request health metrics (heart rate, SpO2)
meshtastic-cli request telemetry --dest 04e1c43b --type health

# Request host metrics (free memory, disk, load average)
meshtastic-cli request telemetry --dest 04e1c43b --type host
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |
| `--type` | Telemetry type: `device`, `environment`, `air-quality`, `power`, `local-stats`, `health`, `host` (default: `device`) |
| `--timeout` | Timeout in seconds (default: 30) |

#### `request position`

Request position data from a remote node.

```bash
# Request by node ID
meshtastic-cli request position --dest 04e1c43b
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |

#### `request metadata`

Request device metadata (firmware version, hardware model, capabilities) from a remote node.

```bash
# Request by node ID
meshtastic-cli request metadata --dest 04e1c43b

# Request by name with custom timeout
meshtastic-cli request metadata --to Pedro --timeout 60
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |
| `--timeout` | Seconds to wait for response (default: 30) |

Example output:

```
Device metadata from Pedro (!04e1c43b):
  Firmware:     2.5.6.abc1234
  Hardware:     HELTEC_V3
  Device ID:    04e1c43b
  Capabilities: HasWifi, HasBluetooth
```

### `gpio`

Remote GPIO pin operations on mesh nodes. Requires the target node to have the remote hardware module enabled. GPIO mask values can be provided in decimal or `0x` hex format.

#### `gpio write`

Write a value to GPIO pins on a remote node. The mask specifies which pins to affect; the value specifies the state to write to those pins.

```bash
# Set GPIO pin 4 high on a remote node (mask and value in decimal)
meshtastic-cli gpio write --dest 04e1c43b --mask 16 --value 16

# Set GPIO pin 4 high (mask and value in hex)
meshtastic-cli gpio write --dest 04e1c43b --mask 0x10 --value 0x10

# Set pin 4 high and pin 5 low
meshtastic-cli gpio write --to Pedro --mask 0x30 --value 0x10
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |
| `--mask` | Bitmask of GPIO pins to write (decimal or 0x hex) |
| `--value` | Values to write to the masked pins (decimal or 0x hex) |

#### `gpio read`

Read the current state of GPIO pins from a remote node.

```bash
# Read pins 4 and 5 from a remote node (mask in decimal)
meshtastic-cli gpio read --dest 04e1c43b --mask 48

# Read using hex mask
meshtastic-cli gpio read --to Pedro --mask 0x30

# Custom timeout
meshtastic-cli gpio read --dest 04e1c43b --mask 0x10 --timeout 60
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |
| `--mask` | Bitmask of GPIO pins to read (decimal or 0x hex) |
| `--timeout` | Seconds to wait for the response (default: 30) |

Example output:

```
GPIO state from Pedro (!04e1c43b):
  Mask:  0x00000030
  Value: 0x00000010  (pin 4: HIGH, pin 5: LOW)
```

#### `gpio watch`

Watch for GPIO state changes on a remote node. Runs continuously until interrupted with Ctrl+C. Each state change is printed with a timestamp.

```bash
# Watch pins 4 and 5
meshtastic-cli gpio watch --dest 04e1c43b --mask 0x30

# Watch by node name
meshtastic-cli gpio watch --to Pedro --mask 0x10
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |
| `--mask` | Bitmask of GPIO pins to watch (decimal or 0x hex) |

Example output:

```
-> Watching GPIO on Pedro (!04e1c43b) [mask: 0x00000030]. Press Ctrl+C to stop.

[15:30:02] Value changed: 0x00000010  (pin 4: HIGH, pin 5: LOW)
[15:31:15] Value changed: 0x00000030  (pin 4: HIGH, pin 5: HIGH)
[15:32:40] Value changed: 0x00000000  (pin 4: LOW, pin 5: LOW)
```

### `completions`

Generate shell completion scripts. Once installed, completions enable tab completion for all commands, subcommands, flags, and many argument values directly in your shell.

```bash
# Print completion script for the current shell to stdout
meshtastic-cli completions bash
meshtastic-cli completions zsh
meshtastic-cli completions fish
meshtastic-cli completions powershell
meshtastic-cli completions elvish
```

#### Installing completions

```bash
# bash — add to your shell profile or drop into the completions directory
meshtastic-cli completions bash > ~/.local/share/bash-completion/completions/meshtastic-cli

# zsh — place in a directory on your $fpath
meshtastic-cli completions zsh > ~/.zfunc/_meshtastic-cli
# Then add to ~/.zshrc if not already present:
# fpath=(~/.zfunc $fpath)
# autoload -Uz compinit && compinit

# fish
meshtastic-cli completions fish > ~/.config/fish/completions/meshtastic-cli.fish
```

| Option | Description |
|---|---|
| `<SHELL>` | Target shell: `bash`, `zsh`, `fish`, `powershell`, `elvish` (required) |

### `config-file`

Manage a persistent configuration file stored at `~/.config/meshtastic-cli/config.toml`. Values set here are applied automatically on every invocation, so you do not have to repeat connection options or other defaults on every command. Command-line flags always override config file values.

```bash
# Show current config file contents
meshtastic-cli config-file show

# Print the path to the config file
meshtastic-cli config-file path

# Set a persistent default value
meshtastic-cli config-file set host 192.168.1.100
meshtastic-cli config-file set port 4403
meshtastic-cli config-file set serial /dev/ttyUSB0

# Remove a previously set value (revert to built-in default)
meshtastic-cli config-file unset host
meshtastic-cli config-file unset serial
```

Example config file (`~/.config/meshtastic-cli/config.toml`):

```toml
host = "192.168.1.100"
port = 4403
```

Available keys:

| Key | Description | Equivalent flag |
|---|---|---|
| `host` | Default TCP host | `--host` |
| `port` | Default TCP port | `--port` |
| `serial` | Default serial device path | `--serial` |

| Subcommand | Description |
|---|---|
| `show` | Print the current config file contents as TOML |
| `set <KEY> <VALUE>` | Set a persistent default value |
| `unset <KEY>` | Remove a key, reverting to the built-in default |
| `path` | Print the filesystem path of the config file |

### `waypoint`

Send, delete, and list waypoints on the mesh network. Waypoints are named geographic points that appear on the map in compatible Meshtastic clients.

```bash
# Send a waypoint broadcast to all nodes
meshtastic-cli waypoint send --name "Base Camp" --lat 40.4168 --lon -3.7038

# Send a waypoint with full options
meshtastic-cli waypoint send \
  --name "Checkpoint A" \
  --lat 40.4168 \
  --lon -3.7038 \
  --alt 650 \
  --icon 9410 \
  --expire 3600 \
  --dest 04e1c43b

# Delete a waypoint by ID
meshtastic-cli waypoint delete --id 42

# List all waypoints known to the local node
meshtastic-cli waypoint list
```

| Subcommand | Description |
|---|---|
| `send` | Broadcast or unicast a new waypoint |
| `delete` | Delete a waypoint by its numeric ID |
| `list` | List all waypoints currently stored on the local node |

`waypoint send` options:

| Option | Description |
|---|---|
| `--name` | Waypoint name, up to 30 characters (required) |
| `--lat` | Latitude in decimal degrees (required) |
| `--lon` | Longitude in decimal degrees (required) |
| `--alt` | Altitude in meters (optional) |
| `--icon` | Unicode code point for the waypoint icon displayed in clients (optional) |
| `--expire` | Seconds until the waypoint expires (optional, omit for no expiry) |
| `--dest` | Target node ID in hex for a unicast waypoint (omit to broadcast) |
| `--to` | Target node name for a unicast waypoint (omit to broadcast) |

`waypoint delete` options:

| Option | Description |
|---|---|
| `--id` | Numeric waypoint ID to delete (required) |

### `watch`

Displays the node table as a live-updating view that refreshes periodically in place, similar to the `watch` Unix utility applied to the `nodes` output. Press Ctrl+C to stop.

```bash
# Watch node table, refresh every 30 seconds (default)
meshtastic-cli watch

# Refresh every 10 seconds
meshtastic-cli watch --interval 10

# Watch with a custom field set
meshtastic-cli watch --fields id,name,battery,snr --interval 15
```

| Option | Description |
|---|---|
| `--interval` | Refresh interval in seconds (default: 30) |
| `--fields` | Comma-separated list of columns to display (same values as `nodes --fields`) |

Example output (refreshes in place):

```
meshtastic-cli watch  --  refreshing every 30s  --  last update: 15:30:00  --  Ctrl+C to stop

  ID          Name          Battery   SNR     Hops   Last Heard
  !04e1c43b   Pedro         85%       8.5     0      just now
  !a1b2c3d4   Maria         72%       6.0     1      2m ago
  !e5f6a7b8   Relay-1       --        4.5     2      5m ago
```

### `mqtt bridge`

Bidirectional MQTT bridge. Subscribes to incoming mesh packets and republishes them to an MQTT broker as JSON, and optionally subscribes to an MQTT topic to inject messages back into the mesh. Useful for integrating a Meshtastic mesh into home automation, dashboards, or data pipelines without enabling the built-in MQTT module on the device.

```bash
# Bridge to a local MQTT broker with default topic prefix
meshtastic-cli mqtt bridge --broker mqtt://localhost:1883

# Bridge with authentication
meshtastic-cli mqtt bridge \
  --broker mqtt://broker.example.com:1883 \
  --username myuser \
  --password mypassword

# Bridge with a custom topic prefix (default: meshtastic)
meshtastic-cli mqtt bridge \
  --broker mqtt://localhost:1883 \
  --topic my-mesh

# Bridge without bidirectional injection (publish only)
meshtastic-cli mqtt bridge \
  --broker mqtt://localhost:1883 \
  --no-downlink
```

Published topic format:

```
<prefix>/<node-id>/<port-name>
```

Example topics published by the bridge:

```
meshtastic/04e1c43b/text
meshtastic/04e1c43b/position
meshtastic/04e1c43b/telemetry/device
```

Each message is a JSON object containing the decoded packet fields plus metadata (timestamp, sender node ID, SNR, RSSI, hops).

Downlink topic (for injecting messages into the mesh):

```
<prefix>/downlink/send
```

Post a JSON payload to this topic to send a text message via the bridge:

```json
{ "text": "hello mesh", "channel": 0 }
```

| Option | Description |
|---|---|
| `--broker` | MQTT broker URL including scheme and port, e.g. `mqtt://localhost:1883` (required) |
| `--username` | MQTT username for authenticated brokers (optional) |
| `--password` | MQTT password for authenticated brokers (optional) |
| `--topic` | Topic prefix for all published messages (default: `meshtastic`) |
| `--no-downlink` | Disable the downlink subscription (publish-only mode) |

### `shell`

Interactive REPL (Read-Eval-Print Loop) for exploratory and interactive use. The shell maintains a single persistent connection to the device for the duration of the session, avoiding the startup overhead of reconnecting for every command. Commands are the same as in non-interactive mode; the connection flags (`--host`, `--serial`, `--ble`) are specified once when launching the shell.

```bash
# Start an interactive shell (connects to default TCP host)
meshtastic-cli shell

# Start an interactive shell connected to a serial device
meshtastic-cli --serial /dev/ttyUSB0 shell
```

Features:

- Command history persisted to `~/.local/share/meshtastic-cli/history` across sessions
- Tab completion for all commands, subcommands, and flags (powered by `rustyline`)
- Single device connection reused for the entire session
- `help` prints available commands
- `exit` or Ctrl+D to quit

Example session:

```
meshtastic-cli> nodes
  ID          Name    Battery  SNR    Hops  Last Heard
  !04e1c43b   Pedro   85%      8.5    0     just now
  !a1b2c3d4   Maria   72%      6.0    1     2m ago

meshtastic-cli> send "hello from shell"
ok Message sent.

meshtastic-cli> ping --to Maria
-> Pinging !a1b2c3d4 (Maria) (packet id: 7f3a1b2c)...
ok ACK from !a1b2c3d4 (Maria) in 1.8s

meshtastic-cli> exit
Goodbye.
```

---

## Architecture

### System Design

```
CLI Input
    |
    v
main.rs  (argument parsing + dispatch only)
    |
    +---> connection.rs  (TCP, Serial, or BLE -> StreamApi)
    |
    +---> config_file.rs  (persistent CLI config at ~/.config/meshtastic-cli/config.toml)
    |
    +---> commands/
              mod.rs          (Command trait definition)
              nodes.rs        (implements Command for node listing)
              send.rs         (implements Command for sending messages)
              listen.rs       (implements Command for packet streaming)
              info.rs         (implements Command for device info display)
              ping.rs         (implements Command for node ping with ACK)
              config.rs       (implements Command for config get/set)
              channel.rs      (implements Command for channel management)
              traceroute.rs   (implements Command for route tracing)
              export_import.rs (implements Command for config export/import)
              device.rs       (implements Command for reboot/shutdown/time/canned/ringtone)
              node.rs         (implements Command for node management)
              position.rs     (implements Command for GPS position get/set/remove)
              request.rs      (implements Command for remote data requests)
              reply.rs        (implements Command for auto-reply)
              gpio.rs         (implements Command for remote GPIO operations)
              support.rs      (implements Command for diagnostic info display)
              waypoint.rs     (implements Command for waypoint send/delete/list)
              watch.rs        (implements Command for live-updating node table)
              mqtt_bridge.rs  (implements Command for bidirectional MQTT bridge)
              shell.rs        (implements Command for interactive REPL)
```

### Key Patterns

- **Command pattern (Strategy)**: `commands/mod.rs` defines a `Command` trait. Each subcommand implements it independently. `main.rs` dispatches to the correct implementor based on parsed CLI input.
- **Connection abstraction**: `connection.rs` encapsulates TCP (via `meshtastic`'s `StreamApi`), serial (via `tokio-serial`), and BLE connections, exposing a unified interface to commands.
- **Error types**: `error.rs` uses `thiserror` for structured, typed errors. `anyhow` is used at the boundary (main) for ergonomic top-level error handling.
- **Feature flags**: BLE support is gated behind the `ble` Cargo feature to avoid requiring Bluetooth platform libraries in environments that do not need them.
- **Persistent config**: `config_file.rs` reads `~/.config/meshtastic-cli/config.toml` at startup and merges stored defaults with command-line flags before dispatch, following standard XDG conventions.

### Tech Stack

| Component       | Crate / Tool           | Reason                                              |
|-----------------|------------------------|-----------------------------------------------------|
| Language        | Rust 2021              | Safety, performance, strong async ecosystem         |
| Async runtime   | Tokio                  | Required by the `meshtastic` crate                  |
| Device protocol | meshtastic v0.1.8      | Official Rust crate for Meshtastic protocol         |
| CLI parsing     | clap (derive)          | Ergonomic, zero-boilerplate argument definitions    |
| Shell completions | clap_complete        | Generate shell completions from clap definitions    |
| Error handling  | thiserror / anyhow     | Typed errors in libraries, ergonomic in binaries    |
| Serial I/O      | tokio-serial           | Async serial port support                           |
| Terminal output | colored                | Readable, colored CLI output                        |
| Terminal UI     | crossterm              | Terminal manipulation for the live `watch` display  |
| Serialization   | serde / serde_yaml     | YAML config export and import                       |
| JSON output     | serde_json             | Structured JSON output for `--json` flag            |
| Config file     | toml / dirs            | Persistent CLI config file parsing and XDG paths    |
| QR codes        | qrcode                 | QR code generation for terminal, PNG, and SVG       |
| Image output    | image                  | PNG image rendering for QR code export              |
| MQTT client     | rumqttc                | Async MQTT client for the bridge command            |
| Interactive REPL | rustyline / shlex     | Command history, line editing, and tab completion for `shell` |

> Note: The `meshtastic` crate (v0.1.8) is early-stage. When something appears underdocumented, refer to the source: https://github.com/meshtastic/rust

---

## Development

### Build

```bash
cargo build            # debug build
cargo build --release  # optimized release build

# With BLE support
cargo build --features ble
cargo build --release --features ble
```

### Run (without installing)

```bash
# TCP — local simulator
cargo run -- --host 127.0.0.1 --port 4403 nodes

# Serial
cargo run -- --serial /dev/ttyUSB0 nodes

# BLE (requires --features ble build)
cargo run --features ble -- --ble "Meshtastic_abcd" nodes
```

### Tests

```bash
cargo test                   # run all tests
cargo test <test_name>       # run a single test by name
```

### Lint and Format

```bash
cargo clippy -- -D warnings  # lint; treats warnings as errors
cargo fmt --check            # check formatting without applying
cargo fmt                    # apply formatting
```

---

## Project Structure

```
meshtastic-cli/
├── Cargo.toml
├── Cargo.lock
├── config.yaml              # Docker simulator config
├── README.md
├── CHANGELOG.md
└── src/
    ├── main.rs              # CLI parsing and command dispatch only
    ├── cli.rs               # Clap argument and subcommand definitions
    ├── connection.rs        # TCP, serial, and BLE connection handling
    ├── config_file.rs       # Persistent CLI config (~/.config/meshtastic-cli/config.toml)
    ├── error.rs             # Typed error definitions (thiserror)
    ├── node_db.rs           # Node data model and local node database
    ├── router.rs            # Packet routing and dispatch logic
    └── commands/
        ├── mod.rs           # Command trait and module exports
        ├── nodes.rs         # `nodes` command implementation
        ├── send.rs          # `send` command implementation
        ├── listen.rs        # `listen` command implementation
        ├── info.rs          # `info` command implementation
        ├── ping.rs          # `ping` command implementation
        ├── config.rs        # `config get/set/set-ham/set-url` implementation
        ├── traceroute.rs    # `traceroute` command implementation
        ├── channel.rs       # `channel add/del/set/list/qr` implementation
        ├── export_import.rs # `config export`/`config import` implementation
        ├── device.rs        # `device` subcommands implementation
        ├── node.rs          # `node` subcommands implementation
        ├── position.rs      # `position get/set/remove` implementation
        ├── request.rs       # `request telemetry/position/metadata` implementation
        ├── reply.rs         # `reply` command implementation
        ├── gpio.rs          # `gpio write/read/watch` implementation
        ├── support.rs       # `support` command implementation
        ├── waypoint.rs      # `waypoint send/delete/list` implementation
        ├── watch.rs         # `watch` live node table implementation
        ├── mqtt_bridge.rs   # `mqtt bridge` bidirectional bridge implementation
        └── shell.rs         # `shell` interactive REPL implementation
```

---

## Roadmap

| Command | Description | Status |
|---|---|---|
| `nodes` | List all mesh nodes with device and signal info | v0.1.0 |
| `send <msg>` | Send a text message to the mesh | v0.2.0 |
| `listen` | Stream all incoming packets to stdout in real time | Done |
| `info` | Show local node info: ID, firmware version, channels | Done |
| `ping <node-id>` | Send a ping to a specific node and wait for ACK | Done |
| `config get/set/export/import` | Read, write, export, and import device configuration | Done |

### Tier 1 — High Priority

| Command | Description | Status |
|---|---|---|
| `traceroute` | Trace route to a node showing each hop with SNR | Done |
| `channel add/del/set` | Add, delete, and configure channels | Done |
| `config export` | Export full device config as YAML | Done |
| `config import` | Import and apply config from a YAML file | Done |
| `device reboot` | Reboot a node (local or remote) | Done |
| `device shutdown` | Shut down a node (local or remote) | Done |
| `node set-owner` | Set device long name and short name | Done |

### Tier 2 — Medium Priority

| Command | Description | Status |
|---|---|---|
| `request telemetry` | Request telemetry from a remote node | Done |
| `request position` | Request position from a remote node | Done |
| `position set` | Set fixed GPS position (lat, lon, alt) | Done |
| `device factory-reset` | Restore default device configuration | Done |
| `device reset-nodedb` | Clear the node's entire NodeDB | Done |
| `node remove` | Remove a specific node from NodeDB | Done |
| `config set-ham` | Set licensed Ham radio callsign | Done |
| `config set-url` | Set channels and LoRa config from a meshtastic URL | Done |

### Tier 3 — Lower Priority

| Command | Description | Status |
|---|---|---|
| `channel qr` | Show QR code and URL for channel sharing | Done |
| `device set-canned-message` | Set canned messages on the device | Done |
| `device get-canned-message` | Display configured canned messages | Done |
| `device set-ringtone` | Set notification ringtone in RTTTL format | Done |
| `node set-favorite` / `remove-favorite` | Mark/unmark a node as favorite | Done |
| `node set-ignored` / `remove-ignored` | Mark/unmark a node as ignored | Done |
| `device set-time` | Set node time via Unix timestamp | Done |
| `request metadata` | Retrieve device metadata from a remote node | Done |

### Feature Parity Additions

| Feature | Description | Status |
|---|---|---|
| `reply` | Auto-reply to received messages with signal info | Done |
| `gpio write/read/watch` | Remote GPIO pin operations on mesh nodes | Done |
| `support` | Display diagnostic info about device and CLI | Done |
| `send --ack` | Wait for delivery confirmation before returning | Done |
| `send --private` | Send on PRIVATE_APP port for private messaging | Done |
| `channel qr --all` | Generate individual QR per active channel | Done |
| `config begin-edit` / `commit-edit` | Batch config change signaling | Done |
| `config set-modem-preset` | Set modem preset directly by name | Done |
| `config ch-add-url` | Add channels from URL without replacing existing | Done |
| `nodes --fields` | Select which columns to display | Done |
| `--ble` / `--ble-scan` | BLE connection support | Done |
| `--no-nodes` | Skip node collection on startup for faster startup | Done |
| `position remove` | Clear fixed GPS position | Done |
| `position set --flags` | Set position broadcast field flags | Done |
| `device get-ringtone` | Display the stored ringtone | Done |
| `device reboot-ota` | Reboot into OTA firmware update mode (ESP32) | Done |
| `device enter-dfu` | Enter DFU mode (NRF52 devices) | Done |
| `device factory-reset-device` | Full factory reset including BLE bond wipe | Done |
| `request telemetry --type` | Select telemetry variant (device, environment, air-quality, power, local-stats, health, host) | Done |
| `node set-unmessageable` | Mark local node as unmessageable/messageable | Done |
| Named position flags | Accept flag names (ALTITUDE, TIMESTAMP, etc.) in addition to numeric bitmask | Done |

### Post-Parity Enhancements

| Feature | Description | Status |
|---|---|---|
| `--json` | Global flag for structured JSON output on all data-returning commands | Done |
| `completions` | Generate shell completion scripts for bash, zsh, fish, PowerShell, Elvish | Done |
| `config-file` | Persistent CLI config at `~/.config/meshtastic-cli/config.toml` (show, set, unset, path) | Done |
| `waypoint send/delete/list` | Waypoint management on the mesh (name, lat/lon/alt, icon, expiry, unicast) | Done |
| `watch` | Live-updating node table with configurable refresh interval | Done |
| `listen --log` | Write received packets as JSON Lines to a log file | Done |
| `mqtt bridge` | Bidirectional MQTT bridge with configurable topic prefix and downlink injection | Done |
| `shell` | Interactive REPL with persistent command history and tab completion | Done |

---

## Contributing

Contributions are welcome. Please ensure:

- All code follows SOLID principles — one responsibility per module, depend on abstractions
- New commands are added as independent modules under `src/commands/`
- `cargo clippy -- -D warnings` passes with no warnings
- `cargo fmt` is applied before committing
- Tests are added for new logic where feasible

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## Project Status

**Current Version**: 0.3.0
**Development Status**: Active development
**Stability**: Experimental — API and CLI interface may change

**Next Milestones**:
- Investigate TLS/mTLS support for the MQTT bridge
- Explore structured logging integration (tracing crate) for long-running commands
- Community-requested features and hardware-specific improvements
