# Architecture

## System Design

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

## Key Patterns

- **Command pattern (Strategy)**: `commands/mod.rs` defines a `Command` trait. Each subcommand implements it independently. `main.rs` dispatches to the correct implementor based on parsed CLI input.
- **Connection abstraction**: `connection.rs` encapsulates TCP (via `meshtastic`'s `StreamApi`), serial (via `tokio-serial`), and BLE connections, exposing a unified interface to commands.
- **Error types**: `error.rs` uses `thiserror` for structured, typed errors. `anyhow` is used at the boundary (main) for ergonomic top-level error handling.
- **Feature flags**: BLE support is gated behind the `ble` Cargo feature to avoid requiring Bluetooth platform libraries in environments that do not need them.
- **Persistent config**: `config_file.rs` reads `~/.config/meshtastic-cli/config.toml` at startup and merges stored defaults with command-line flags before dispatch, following standard XDG conventions.

## Tech Stack

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

> Note: The `meshtastic` crate (v0.1.8) is early-stage. When something appears underdocumented, refer to the source: <https://github.com/meshtastic/rust>

## Project Structure

```
meshtastic-cli/
├── Cargo.toml
├── Cargo.lock
├── config.yaml              # Docker simulator config
├── README.md
├── CHANGELOG.md
├── docs/                    # mdBook documentation (this site)
│   ├── book.toml
│   └── src/
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
