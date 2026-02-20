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

## Features (v0.1.0)

- TCP connectivity to local simulators or remote devices
- Serial connectivity to physical Meshtastic hardware
- `nodes` command: list all mesh nodes with ID, name, battery level, SNR, hop count, and last-heard timestamp
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
              send.rs     (planned)
              listen.rs   (planned)
              info.rs     (planned)
              ping.rs     (planned)
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
        └── nodes.rs         # `nodes` command implementation
```

---

## Roadmap

The following commands are planned in priority order:

| Command         | Description                                           | Status     |
|-----------------|-------------------------------------------------------|------------|
| `nodes`         | List all mesh nodes with device and signal info       | v0.1.0     |
| `send <msg>`    | Send a text message to the mesh                       | Planned    |
| `listen`        | Stream all incoming packets to stdout in real time    | Planned    |
| `info`          | Show local node info: ID, firmware version, channels  | Planned    |
| `ping <node-id>`| Send a ping to a specific node and wait for ACK       | Planned    |

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

**Current Version**: 0.1.0
**Development Status**: Early development
**Stability**: Experimental — API and CLI interface may change

**Next Milestones**:
- `send` command for broadcasting text messages to the mesh
- `listen` command for real-time packet streaming
- `info` command for local node metadata
- `ping` command with ACK waiting
