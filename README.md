# meshtastic-cli

> A Rust CLI tool for interacting with Meshtastic mesh networking devices over TCP, serial, or BLE connections.

## Quick Start

```bash
# Download the latest binary for your platform (includes BLE support)
# → https://github.com/matutetandil/meshtastic-cli/releases

meshtastic-cli --host 192.168.1.100 nodes   # list nodes on your mesh
meshtastic-cli send "hello mesh"            # send a message to the network
```

That's it — connect, see your mesh, send a message.

**[Full Documentation](https://matutetandil.github.io/meshtastic-cli)** — complete guides, all command references, and architecture details.

---

## Purpose & Context

**What it does**: `meshtastic-cli` provides a command-line interface to Meshtastic devices, allowing you to list nodes, send messages, monitor incoming packets, query device info, ping specific nodes, manage channels, control GPIO pins, and more — all from a terminal.

**Why it exists**: The Meshtastic ecosystem lacks a robust, composable CLI tool built in Rust. This project aims to fill that gap as an open-source contribution, leveraging the official [`meshtastic` Rust crate](https://github.com/meshtastic/rust) to interact with real hardware and local simulators alike.

**Who it's for**: Developers and operators working with Meshtastic mesh networks who want scriptable, terminal-native access to device data without a GUI.

---

## Features (v0.4.0)

| Category | Highlights |
|---|---|
| **Connectivity** | TCP, serial, BLE; `--no-nodes` for fast startup; persistent config file |
| **Messaging** | `send` (broadcast/targeted/ACK/private), `listen` (with `--log`), `reply` (auto-reply with signal info) |
| **Node Info** | `nodes` (with `--fields`), `info`, `support`, `watch` (live TUI) |
| **Configuration** | `config get/set/export/import`, `set-ham`, `set-url`, `begin-edit/commit-edit`, `set-modem-preset`, `ch-add-url` |
| **Channels** | `channel add/del/set/list/qr` (QR to terminal, PNG, SVG; `--all` per-channel) |
| **Device Mgmt** | `reboot`, `shutdown`, `reboot-ota`, `enter-dfu`, `factory-reset`, `set-time`, canned messages, ringtone |
| **Node Mgmt** | `set-owner`, `remove`, `set-favorite`, `set-ignored`, `set-unmessageable` |
| **Position** | `position get/set/remove` with named broadcast flags |
| **Remote Data** | `request telemetry` (7 types), `position`, `metadata` |
| **Network Diag** | `ping` (RTT), `traceroute` (hop + SNR) |
| **GPIO** | `gpio write/read/watch` on remote nodes |
| **Waypoints** | `waypoint send/delete/list` |
| **Integration** | `mqtt bridge` (bidirectional mesh-to-MQTT), `shell` (REPL with tab completion) |
| **Output** | `--json` global flag, `completions` for bash/zsh/fish/PowerShell/Elvish |

---

## Installation

### Pre-built binaries (recommended)

Download the latest binary from [GitHub Releases](https://github.com/matutetandil/meshtastic-cli/releases):

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
```

### Install from crates.io

```bash
cargo install meshtastic-cli
```

### Build from source

```bash
git clone https://github.com/matutetandil/meshtastic-cli.git
cd meshtastic-cli
cargo build --release
```

### BLE support

All pre-built binaries include BLE support. When building from source:

```bash
cargo build --release --features ble
```

Linux requires BlueZ: `sudo apt install libbluetooth-dev`

---

## Command Reference

| Command | Description | Docs |
|---|---|---|
| `nodes` | List all mesh nodes with signal and device info | [Messaging](https://matutetandil.github.io/meshtastic-cli/commands/messaging.html) |
| `send` | Send text messages (broadcast, targeted, ACK, private) | [Messaging](https://matutetandil.github.io/meshtastic-cli/commands/messaging.html) |
| `listen` | Stream incoming packets in real time | [Messaging](https://matutetandil.github.io/meshtastic-cli/commands/messaging.html) |
| `reply` | Auto-reply with signal info (SNR, RSSI, hops) | [Messaging](https://matutetandil.github.io/meshtastic-cli/commands/messaging.html) |
| `info` | Display local node and device details | [Messaging](https://matutetandil.github.io/meshtastic-cli/commands/messaging.html) |
| `support` | Diagnostic summary for bug reports | [Messaging](https://matutetandil.github.io/meshtastic-cli/commands/messaging.html) |
| `ping` | Ping a node, measure round-trip time | [Network](https://matutetandil.github.io/meshtastic-cli/commands/network.html) |
| `traceroute` | Trace route with SNR per hop | [Network](https://matutetandil.github.io/meshtastic-cli/commands/network.html) |
| `config` | Get/set/export/import device configuration | [Config](https://matutetandil.github.io/meshtastic-cli/commands/config.html) |
| `channel` | Add, delete, set, list, QR code | [Channel](https://matutetandil.github.io/meshtastic-cli/commands/channel.html) |
| `device` | Reboot, shutdown, factory reset, time, ringtone | [Device](https://matutetandil.github.io/meshtastic-cli/commands/device.html) |
| `node` | Set owner, remove, favorite, ignored | [Node](https://matutetandil.github.io/meshtastic-cli/commands/node.html) |
| `position` | Get, set, remove GPS position | [Position](https://matutetandil.github.io/meshtastic-cli/commands/position.html) |
| `request` | Request telemetry, position, metadata | [Request](https://matutetandil.github.io/meshtastic-cli/commands/request.html) |
| `gpio` | Remote GPIO write, read, watch | [GPIO](https://matutetandil.github.io/meshtastic-cli/commands/gpio.html) |
| `waypoint` | Send, delete, list waypoints | [Waypoint](https://matutetandil.github.io/meshtastic-cli/commands/waypoint.html) |
| `watch` | Live-updating node table | [Watch](https://matutetandil.github.io/meshtastic-cli/commands/watch.html) |
| `mqtt bridge` | Bidirectional MQTT bridge | [MQTT](https://matutetandil.github.io/meshtastic-cli/commands/mqtt-bridge.html) |
| `shell` | Interactive REPL with tab completion | [Shell](https://matutetandil.github.io/meshtastic-cli/commands/shell.html) |
| `completions` | Generate shell completion scripts | [Completions](https://matutetandil.github.io/meshtastic-cli/commands/completions.html) |
| `config-file` | Manage persistent CLI config | [Config File](https://matutetandil.github.io/meshtastic-cli/commands/config-file.html) |

---

## Connection Examples

```bash
# TCP (default: localhost:4403, ideal for Docker simulator)
meshtastic-cli nodes

# TCP with custom host
meshtastic-cli --host 192.168.1.100 nodes

# Serial
meshtastic-cli --serial /dev/ttyUSB0 nodes

# BLE by name
meshtastic-cli --ble "Meshtastic_abcd" nodes

# BLE scan
meshtastic-cli --ble-scan

# Skip node discovery for faster startup
meshtastic-cli --no-nodes send "hello mesh"

# JSON output for scripting
meshtastic-cli --json nodes | jq '.[].name'
```

---

## Contributing

Contributions are welcome. Please ensure `cargo clippy -- -D warnings` and `cargo fmt --check` pass before submitting. See the [Contributing Guide](https://matutetandil.github.io/meshtastic-cli/contributing.html) for details.

---

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

## Project Status

**Current Version**: 0.4.0
**Development Status**: Active development
**Stability**: Experimental — API and CLI interface may change

---

## Support

If you find this project useful, consider supporting its development:

<a href="https://buymeacoffee.com/matutetandil" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-yellow.png" alt="Buy Me A Coffee" width="200"></a>
