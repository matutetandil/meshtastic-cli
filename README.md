# meshtastic-cli

> A Rust CLI tool for interacting with Meshtastic mesh networking devices over TCP or serial connections.

## Purpose & Context

**What it does**: `meshtastic-cli` provides a command-line interface to Meshtastic devices, allowing you to list nodes, send messages, monitor incoming packets, query device info, and ping specific nodes — all from a terminal.

**Why it exists**: The Meshtastic ecosystem lacks a robust, composable CLI tool built in Rust. This project aims to fill that gap as an open-source contribution, leveraging the official `meshtastic` Rust crate to interact with real hardware and local simulators alike.

**Who it's for**: Developers and operators working with Meshtastic mesh networks who want scriptable, terminal-native access to device data without a GUI.

**Key Design Decisions**:
- Strategy pattern for commands: each command is an independent module implementing a shared trait, making it trivial to add new commands without touching existing ones.
- SOLID principles throughout: single responsibility per module, open/closed for command extension, dependency inversion via connection abstraction.
- Thin `main.rs`: only parses CLI arguments and dispatches to the appropriate command — no business logic lives there.
- Async-first: all I/O uses Tokio, matching the async model of the underlying `meshtastic` crate.

---

## Features (v0.2.0)

- TCP connectivity to local simulators or remote devices
- Serial connectivity to physical Meshtastic hardware
- `nodes` command: list all mesh nodes with ID, name, battery level, SNR, hop count, and last-heard timestamp
- `send` command: send text messages to the mesh (broadcast, by node ID, by node name, or on a specific channel)
- `listen` command: stream and decode incoming packets in real time (text, position, telemetry, routing, node info)
- `info` command: display local node details, firmware, capabilities, channels, device metrics, and position
- `ping` command: ping a specific node by ID or name, measure round-trip time, with configurable timeout
- `config get` command: display all or individual device/module configuration sections
- `config set` command: modify any configuration field with automatic device reboot
- `traceroute` command: trace route to a node showing each hop with SNR values
- `channel` command: add, delete, list, and set properties on channels (name, PSK, uplink/downlink)
- `config export` command: export full device configuration (config, module config, channels) to YAML
- `config import` command: import and apply configuration from a YAML file
- `device reboot` command: reboot local or remote device with configurable delay
- `device shutdown` command: shut down local or remote device with configurable delay
- `node set-owner` command: set device long name and short name
- `device factory-reset` command: restore factory defaults
- `device reset-nodedb` command: clear the node database
- `node remove` command: remove a specific node from the local NodeDB
- `position get` command: display current GPS position
- `position set` command: set a fixed GPS position (latitude, longitude, altitude)
- `request telemetry` command: request telemetry from a remote node
- `request position` command: request position from a remote node
- `config set-ham` command: configure licensed Ham radio mode with callsign
- `config set-url` command: apply channels and LoRa config from a meshtastic:// URL
- Colored terminal output for readability
- Docker simulator support for local development without hardware

---

## Prerequisites

- **Rust toolchain** (edition 2021, stable): install via [rustup.rs](https://rustup.rs)
- **Docker** (optional): required only for the local simulator

---

## Installation

### Build from source

```bash
git clone https://github.com/your-username/meshtastic-cli.git
cd meshtastic-cli
cargo build --release
```

The compiled binary will be at `target/release/meshtastic-cli`.

### Install directly with Cargo

```bash
cargo install --path .
```

This places the binary in `~/.cargo/bin/meshtastic-cli`, which should already be in your `PATH` if you installed Rust via rustup.

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

3. List nodes:

```bash
cargo run -- nodes
# or, after installing:
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
  --host <HOST>      TCP host to connect to [default: 127.0.0.1]
  --port <PORT>      TCP port to connect to [default: 4403]
  --serial <PATH>    Serial device path (e.g. /dev/ttyUSB0). Overrides TCP.
  -h, --help         Print help
  -V, --version      Print version

Commands:
  nodes    List all nodes visible on the mesh
  send     Send a text message to the mesh network
  listen   Stream incoming packets in real time
  info     Show local node and device information
  config      Get, set, export, import, set-ham, set-url
  node        Node management (set-owner, remove)
  device      Device management (reboot, shutdown, factory-reset, reset-nodedb)
  channel     Manage channels (add, delete, set, list)
  position    GPS position (get, set)
  request     Request data from remote nodes (telemetry, position)
  traceroute  Trace route to a node showing each hop
  ping        Ping a node and measure round-trip time
```

### Connection examples

```bash
# TCP — default (connects to localhost:4403, ideal for Docker simulator)
meshtastic-cli nodes

# TCP — custom host and port
meshtastic-cli --host 192.168.1.100 --port 4403 nodes

# Serial — physical device
meshtastic-cli --serial /dev/ttyUSB0 nodes
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
```

> **Shell note:** The `!` prefix is optional. If you include it, quote or escape it to prevent shell history expansion: `--dest '!04e1c43b'` or `--dest \!04e1c43b`.

| Option      | Description                                            |
|-------------|--------------------------------------------------------|
| `<MESSAGE>` | The text message to send (required, positional)        |
| `--dest`    | Destination node ID in hex (e.g. `04e1c43b`). The `!` prefix is optional. Cannot be combined with `--to`. |
| `--to`      | Destination node name (e.g. `Pedro`). Searches known nodes by name (case-insensitive). If multiple nodes match, shows the list and asks you to use `--dest` instead. Cannot be combined with `--dest`. |
| `--channel` | Channel index 0-7 (default: 0)                        |

### `listen`

Streams all incoming packets from the mesh network in real time. Runs continuously until interrupted with Ctrl+C.

```bash
meshtastic-cli listen
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

Example output:

```
→ Listening for packets... Press Ctrl+C to stop.

[15:30:00] !04e1c43b (Pedro) → broadcast      | Text: Hello everyone!
[15:30:05] !a1b2c3d4 (María) → !04e1c43b      | Position: 40.41680, -3.70380, 650m, 8 sats
[15:30:10] !04e1c43b (Pedro) → broadcast       | Telemetry: battery 85%, 3.90V, ch_util 12.3%
[15:30:15] !a1b2c3d4 (María) → !04e1c43b      | Routing: ACK
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

### `device`

Device management commands: reboot, shutdown, factory-reset, and reset-nodedb. Reboot and shutdown support targeting the local device (default) or a remote node.

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

Restore the device to factory defaults. This erases all configuration, channels, and stored data.

```bash
meshtastic-cli device factory-reset
```

#### `device reset-nodedb`

Clear the device's entire node database. This removes all known nodes from the local NodeDB.

```bash
meshtastic-cli device reset-nodedb
```

### `channel`

Manage device channels: list, add, delete, and modify properties.

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

Apply channels and LoRa configuration from a meshtastic:// URL. These URLs are typically generated by the Meshtastic app or web client for sharing device configurations.

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
→ Pinging !04e1c43b (Pedro) (packet id: a1b2c3d4)...
✓ ACK from !04e1c43b (Pedro) in 2.3s
```

If the node doesn't respond:

```
→ Pinging !04e1c43b (Pedro) (packet id: a1b2c3d4)...
✗ Timeout after 30s — no ACK from !04e1c43b (Pedro)
```

### `position`

GPS position commands: get and set.

#### `position get`

Display the current GPS position of the local node.

```bash
meshtastic-cli position get
```

#### `position set`

Set a fixed GPS position on the device. Requires latitude and longitude; altitude is optional.

```bash
# Set position with latitude and longitude
meshtastic-cli position set 40.4168 -3.7038

# Set position with altitude (in meters)
meshtastic-cli position set 40.4168 -3.7038 650
```

| Option | Description |
|---|---|
| `<LATITUDE>` | Latitude in decimal degrees (required) |
| `<LONGITUDE>` | Longitude in decimal degrees (required) |
| `<ALTITUDE>` | Altitude in meters (optional) |

### `request`

Request data from remote nodes.

#### `request telemetry`

Request telemetry data (battery, voltage, channel utilization, etc.) from a remote node.

```bash
# Request by node ID
meshtastic-cli request telemetry --dest 04e1c43b

# Request by name
meshtastic-cli request telemetry --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |

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

---

## Architecture

### System Design

```
CLI Input
    |
    v
main.rs  (argument parsing + dispatch only)
    |
    +---> connection.rs  (TCP or Serial → StreamApi)
    |
    +---> commands/
              mod.rs      (Command trait definition)
              nodes.rs    (implements Command for node listing)
              send.rs     (implements Command for sending messages)
              listen.rs   (implements Command for packet streaming)
              info.rs     (implements Command for device info display)
              ping.rs     (implements Command for node ping with ACK)
              config.rs   (implements Command for config get/set)
              channel.rs  (implements Command for channel management)
              traceroute.rs (implements Command for route tracing)
              export_import.rs (implements Command for config export/import)
              device.rs   (implements Command for reboot/shutdown)
              node.rs     (implements Command for node management)
              position.rs (implements Command for GPS position get/set)
              request.rs  (implements Command for remote data requests)
```

### Key Patterns

- **Command pattern (Strategy)**: `commands/mod.rs` defines a `Command` trait. Each subcommand implements it independently. `main.rs` dispatches to the correct implementor based on parsed CLI input.
- **Connection abstraction**: `connection.rs` encapsulates both TCP (via `meshtastic`'s `StreamApi`) and serial (via `tokio-serial`) connections, exposing a unified interface to commands.
- **Error types**: `error.rs` uses `thiserror` for structured, typed errors. `anyhow` is used at the boundary (main) for ergonomic top-level error handling.

### Tech Stack

| Component       | Crate / Tool           | Reason                                              |
|-----------------|------------------------|-----------------------------------------------------|
| Language        | Rust 2021              | Safety, performance, strong async ecosystem         |
| Async runtime   | Tokio                  | Required by the `meshtastic` crate                  |
| Device protocol | meshtastic v0.1.8      | Official Rust crate for Meshtastic protocol         |
| CLI parsing     | clap (derive)          | Ergonomic, zero-boilerplate argument definitions    |
| Error handling  | thiserror / anyhow     | Typed errors in libraries, ergonomic in binaries    |
| Serial I/O      | tokio-serial           | Async serial port support                           |
| Terminal output | colored                | Readable, colored CLI output                        |
| Serialization   | serde / serde_yaml     | YAML config export and import                       |

> Note: The `meshtastic` crate (v0.1.8) is early-stage. When something appears underdocumented, refer to the source: https://github.com/meshtastic/rust

---

## Development

### Build

```bash
cargo build            # debug build
cargo build --release  # optimized release build
```

### Run (without installing)

```bash
# TCP — local simulator
cargo run -- --host 127.0.0.1 --port 4403 nodes

# Serial
cargo run -- --serial /dev/ttyUSB0 nodes
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
    ├── connection.rs        # TCP and serial connection handling
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
        ├── config.rs        # `config get/set` command implementation
        ├── traceroute.rs    # `traceroute` command implementation
        ├── channel.rs       # `channel` command implementation
        ├── export_import.rs # `config export`/`config import` implementation
        ├── device.rs        # `device reboot`/`device shutdown` implementation
        ├── node.rs          # `node set-owner` implementation
        ├── position.rs      # `position get/set` implementation
        └── request.rs       # `request telemetry/position` implementation
```

---

## Roadmap

The following commands are planned in priority order:

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
| `qr` | Show QR code for channel sharing | Planned |
| `set-canned-message` | Set/get canned messages | Planned |
| `set-ringtone` | Set/get notification ringtone | Planned |
| `set-favorite-node` | Mark/unmark a node as favorite | Planned |
| `set-ignored-node` | Mark/unmark a node as ignored | Planned |
| `set-time` | Set node time via unix timestamp | Planned |
| `device-metadata` | Retrieve metadata from a remote node | Planned |
| `gpio` | Read/write/watch GPIO on remote nodes | Planned |
| `reply` | Auto-reply to received messages with stats | Planned |

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

**Current Version**: 0.2.0
**Development Status**: Early development
**Stability**: Experimental — API and CLI interface may change

**Next Milestones**:
- Additional commands as needed for feature parity with the official Python CLI
