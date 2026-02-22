# Development

## Build

```bash
cargo build            # debug build
cargo build --release  # optimized release build

# With BLE support
cargo build --features ble
cargo build --release --features ble
```

## Run (without installing)

```bash
# TCP — local simulator
cargo run -- --host 127.0.0.1 --port 4403 nodes

# Serial
cargo run -- --serial /dev/ttyUSB0 nodes

# BLE (requires --features ble build)
cargo run --features ble -- --ble "Meshtastic_abcd" nodes
```

## Tests

```bash
cargo test                   # run all tests
cargo test <test_name>       # run a single test by name
```

## Lint and Format

```bash
cargo clippy -- -D warnings  # lint; treats warnings as errors
cargo fmt --check            # check formatting without applying
cargo fmt                    # apply formatting
```

## Docker Simulator

The repository includes a `config.yaml` for the Meshtastic simulator. Start it with:

```bash
docker run -d --name meshtasticd \
  -v ./config.yaml:/etc/meshtasticd/config.yaml:ro \
  -p 4403:4403 \
  meshtastic/meshtasticd:latest meshtasticd -s
```

Then interact with it using the default TCP connection:

```bash
cargo run -- nodes
cargo run -- send "hello from dev"
cargo run -- listen
```

## Building the Documentation

This documentation is built with [mdBook](https://rust-lang.github.io/mdBook/). To preview it locally:

```bash
# Install mdBook
cargo install mdbook

# Build
mdbook build docs

# Serve with live reload
mdbook serve docs --open
```
