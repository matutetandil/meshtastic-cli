# Installation

## Pre-built binaries (recommended)

Download the latest binary for your platform from [GitHub Releases](https://github.com/matutetandil/mttctl/releases):

| Platform | Binary |
|---|---|
| Linux x86_64 | `mttctl-linux-x86_64` |
| Linux ARM64 | `mttctl-linux-aarch64` |
| macOS Intel | `mttctl-macos-x86_64` |
| macOS Apple Silicon | `mttctl-macos-aarch64` |
| Windows x86_64 | `mttctl-windows-x86_64.exe` |

```bash
# Example: Linux x86_64
curl -L https://github.com/matutetandil/mttctl/releases/latest/download/mttctl-linux-x86_64 -o mttctl
chmod +x mttctl
sudo mv mttctl /usr/local/bin/

# Example: macOS Apple Silicon
curl -L https://github.com/matutetandil/mttctl/releases/latest/download/mttctl-macos-aarch64 -o mttctl
chmod +x mttctl
sudo mv mttctl /usr/local/bin/
```

## Install from crates.io

If you have the Rust toolchain installed:

```bash
cargo install mttctl
```

## Build from source

```bash
git clone https://github.com/matutetandil/mttctl.git
cd mttctl
cargo build --release
```

The compiled binary will be at `target/release/mttctl`.

## BLE support

All pre-built binaries include BLE support out of the box. No extra steps needed.

When building from source, add the `ble` feature flag:

```bash
cargo build --release --features ble
# or
cargo install mttctl --features ble
```

Linux requires BlueZ: `sudo apt install libbluetooth-dev`
