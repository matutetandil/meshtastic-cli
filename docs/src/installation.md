# Installation

## Pre-built binaries (recommended)

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

## Install from crates.io

If you have the Rust toolchain installed:

```bash
cargo install meshtastic-cli
```

## Build from source

```bash
git clone https://github.com/matutetandil/meshtastic-cli.git
cd meshtastic-cli
cargo build --release
```

The compiled binary will be at `target/release/meshtastic-cli`.

## BLE support

All pre-built binaries include BLE support out of the box. No extra steps needed.

When building from source, add the `ble` feature flag:

```bash
cargo build --release --features ble
# or
cargo install meshtastic-cli --features ble
```

Linux requires BlueZ: `sudo apt install libbluetooth-dev`
