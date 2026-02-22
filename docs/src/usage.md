# Usage & Connection

## CLI Overview

```
mttctl [OPTIONS] <COMMAND>

Options:
  --host <HOST>        TCP host to connect to [default: 127.0.0.1]
  --port <PORT>        TCP port to connect to [default: 4403]
  --serial <PATH>      Serial device path (e.g. /dev/ttyUSB0). Overrides TCP.
  --ble <NAME|MAC>     BLE device name or MAC address (requires --features ble build)
  --ble-scan           Scan for nearby BLE Meshtastic devices and list them
  --no-nodes           Skip initial node discovery (saves seconds on large meshes)
  --json               Output results as JSON instead of formatted text
  -h, --help           Print help
  -V, --version        Print version
```

## Connection Modes

### TCP (default)

Connects to a Meshtastic device or simulator via TCP. This is the default mode when no `--serial` or `--ble` flag is provided.

```bash
# Default: localhost:4403 (ideal for Docker simulator)
mttctl nodes

# Custom host and port
mttctl --host 192.168.1.100 --port 4403 nodes
```

### Serial

Connect to a physical device over a serial port.

```bash
mttctl --serial /dev/ttyUSB0 nodes
```

### BLE

Connect to a nearby Meshtastic device via Bluetooth Low Energy. Requires the binary to be built with `--features ble`.

```bash
# Connect by device name
mttctl --ble "Meshtastic_abcd" nodes

# Connect by MAC address
mttctl --ble "AA:BB:CC:DD:EE:FF" nodes

# Scan for nearby devices
mttctl --ble-scan
```

## Global Flags

### `--no-nodes`

Skip the initial node discovery phase on startup, which can take several seconds on large meshes. Useful for commands like `send` or `device reboot` that don't need the full node list.

```bash
mttctl --no-nodes send "hello mesh"
```

### `--json`

Output results as a JSON object or array instead of the default formatted text. Useful for shell scripting, log ingestion, or piping output into tools like `jq`.

```bash
# List nodes as JSON
mttctl --json nodes

# Get device config as JSON
mttctl --json config get lora

# Get local node info as JSON
mttctl --json info

# Pipe into jq for filtering
mttctl --json nodes | jq '[.[] | select(.battery < 20)]'
```

The flag is a global option and must be placed before the subcommand name. Commands that produce no structured output (e.g., `send`, `device reboot`) ignore the flag.

## Quick Start with Docker Simulator

The repository includes a `config.yaml` for the Meshtastic simulator. No hardware required:

```bash
# Start the simulator
docker run -d --name meshtasticd \
  -v ./config.yaml:/etc/meshtasticd/config.yaml:ro \
  -p 4403:4403 \
  meshtastic/meshtasticd:latest meshtasticd -s

# List nodes
mttctl nodes
```
