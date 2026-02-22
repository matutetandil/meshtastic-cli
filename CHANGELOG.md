# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Full documentation site built with mdBook, deployed to GitHub Pages at `https://matutetandil.github.io/meshtastic-cli`
- Documentation pages for all commands organized by category: messaging, network, config, channel, device, node, position, request, GPIO, waypoint, watch, MQTT bridge, shell, completions, and config-file
- Architecture, development, installation, usage, and contributing guides
- GitHub Actions workflow for automatic documentation deployment on push to `main`
- `--json` global flag to emit structured JSON output on all data-returning commands: `nodes`, `info`, `support`, `ping`, `send`, `traceroute`, `position get`, `channel list`, `channel qr`, `config get`, `listen`, `reply`, `gpio read`, `gpio watch`, `request telemetry`, `request position`, and `request metadata`
- `completions` command to generate shell completion scripts for bash, zsh, fish, PowerShell, and Elvish; writes to stdout for easy integration with shell startup files
- `config-file` command for managing persistent CLI configuration stored at `~/.config/meshtastic-cli/config.toml`, with four subcommands: `show` (display current config), `set` (write a key/value pair), `unset` (remove a key), and `path` (print the config file location)
- Persistent CLI config file support: values for `host`, `port`, `serial`, `ble`, and `json` are read from `~/.config/meshtastic-cli/config.toml` at startup and merged with command-line flags; CLI flags always take precedence over config file values
- `waypoint send` command to create and broadcast a waypoint to the mesh, with options for `--name`, `--description`, `--icon` (emoji or Unicode codepoint), `--expire` (Unix timestamp or relative duration), and `--locked` (restrict edits to the sending node)
- `waypoint delete` command to remove an existing waypoint by its numeric ID
- `waypoint list` command to listen for incoming `WAYPOINT_APP` packets and display received waypoints, with a configurable `--timeout`
- `watch` command for a live-updating node table that clears and redraws the terminal at a configurable `--interval` (default 5 s), showing the same columns as `nodes` with automatic refresh until interrupted
- `listen --log <file>` flag to write every received packet as a JSON Lines record to a file alongside the existing terminal output, enabling offline analysis without interrupting the live display
- `mqtt bridge` command for bidirectional mesh-to-MQTT bridging: publishes decoded packets to `{prefix}/messages`, `{prefix}/telemetry/{node}`, and `{prefix}/position/{node}` topics, and subscribes to `{prefix}/send` to forward MQTT messages back into the mesh; supports `--broker`, `--port`, `--prefix`, `--username`, and `--password` options
- `shell` command for an interactive REPL that accepts the same subcommands as the top-level CLI; features persistent command history saved at `~/.config/meshtastic-cli/history.txt`, tab completion for command names and subcommands, and a colored prompt displaying the current connection target

### Changed

- README.md condensed from ~2000 lines to ~200 lines as a landing page, with detailed documentation moved to mdBook site
- `Command` trait refactored so `execute` takes `&self` and `&mut CommandContext` instead of `Box<Self>`, enabling commands to be invoked multiple times within a single REPL session without consuming ownership
- `create_command` factory return type updated to `Box<dyn Command + Send>` to satisfy the `Send` bound required for async command dispatch inside the REPL task
- Extracted `format_uptime()` and `hex_decode()` into shared `parsers` module, removing duplication across info, request, channel, and export_import modules
- Separated startup protocol from `NodeDb` into dedicated `node_db_builder` module; `NodeDb` is now a pure data container with constructor
- Shell command names and help descriptions derived dynamically from Clap metadata instead of hard-coded list
- Moved `json` flag from `CommandContext` to individual command structs, decoupling presentation concerns from execution context

### Dependencies Added

- `clap_complete` for shell completion script generation
- `serde_json` for JSON-formatted output across all data-returning commands
- `toml` and `dirs` for persistent config file parsing and platform-appropriate path resolution
- `crossterm` for terminal manipulation (clear-screen, cursor control) used by the `watch` command
- `rumqttc` for async MQTT client support in the `mqtt bridge` command
- `rustyline` for readline-style line editing and persistent history in the `shell` REPL
- `shlex` for shell-style tokenization of REPL input lines before dispatch

## [0.3.0] - 2026-02-20

### Added

- `listen` command to stream and decode incoming mesh packets in real time
- Packet decoding for text messages, position, telemetry (device, environment, power), node info, and routing (ACK/NAK)
- Colored, timestamped output with sender name resolution from node database
- `NodeDb::node_name()` helper for quick node name lookup by ID
- `info` command to display local node details: ID, name, hardware model, role, firmware version, capabilities (WiFi, Bluetooth, Ethernet, PKC), device metrics, GPS position, and active channels with encryption type
- Channel and DeviceMetadata collection during the configure handshake
- `NodeDb::local_node()`, `NodeDb::channels()`, `NodeDb::metadata()`, `NodeDb::my_node_info()` accessors
- `ping` command to measure round-trip time to a specific node via ACK correlation
- Supports `--dest` (hex ID) and `--to` (name lookup) for ping destination
- Configurable `--timeout` (default 30s) for ACK wait
- Shared `DestinationSpec` enum and `resolve_destination()` / `parse_dest_spec()` helpers across send and ping commands
- `config get` command to display all or individual device and module configuration sections (8 device + 13 module sections)
- `config set` command to modify any configuration field via `section.field` key format, with automatic device reboot
- Config and ModuleConfig packet capture during the configure handshake in `NodeDb`
- `NodeDb::local_config()` and `NodeDb::local_module_config()` accessors
- Enum-aware field parsing for config set (case-insensitive, supports both name and raw integer values)
- `ConfigSection` enum with clap `ValueEnum` for tab-completable section names in `config get`
- `traceroute` command to trace the route to a destination node, showing each hop with SNR values
- Supports forward and return path display when paths differ
- Uses `TRACEROUTE_APP` protocol with `RouteDiscovery` protobuf encoding
- `channel list` command to display all channels with role, encryption type, and uplink/downlink status
- `channel add` command to add a secondary channel with configurable PSK (none, default, random, or hex key)
- `channel del` command to delete a channel by index (protects primary channel 0)
- `channel set` command to modify channel properties (name, psk, uplink_enabled, downlink_enabled, position_precision)
- `config export` subcommand to export full device configuration (device config, module config, channels) as YAML to stdout or file
- `config import` subcommand to import and apply configuration from a YAML file, with per-section updates and channel restoration
- `serde` and `serde_yaml` dependencies for YAML serialization/deserialization
- `device reboot` command to reboot local or remote device with configurable delay (default 5s)
- `device shutdown` command to shut down local or remote device with configurable delay (default 5s)
- `DeviceAction` enum with `Reboot` and `Shutdown` subcommands, supporting `--dest`, `--to`, and `--delay` options
- `node set-owner` command to set device long name and short name via `update_user()` API
- `device factory-reset` command to restore device configuration to factory defaults
- `device reset-nodedb` command to clear the local node database
- `node remove` command to remove a specific node from the NodeDB by ID or name
- `position get` command to display current GPS position
- `position set` command to set a fixed GPS position (latitude, longitude, altitude) via `SetFixedPosition` admin message
- `request telemetry` command to request telemetry from a remote node with timeout
- `request position` command to request position from a remote node with timeout
- `config set-ham` command to configure licensed Ham radio mode with callsign, TX power, and frequency
- `config set-url` command to apply channels and LoRa configuration from a meshtastic:// URL
- Auto-generated short name from long name when `--short` is omitted (initials for multi-word, first 4 chars for single word)
- `node set-favorite` command to mark a node as favorite by ID or name
- `node remove-favorite` command to remove a node from the favorites list
- `node set-ignored` command to mark a node as ignored by ID or name
- `node remove-ignored` command to remove a node from the ignored list
- `device set-time` command to synchronize the device clock, using the current system time when no explicit timestamp is provided
- `device set-canned-message` command to configure canned messages on the device, with messages separated by `|`
- `device get-canned-message` command to display the currently configured canned messages
- `device set-ringtone` command to set the device ringtone using RTTTL format
- `request metadata` command to request device metadata (firmware version, hardware model, and capabilities) from a remote node
- `channel qr` command to generate a QR code and shareable URL for the current channel configuration
- `channel qr --output` flag to export QR code as PNG or SVG image file
- `qrcode` dependency for QR code generation (terminal, PNG, SVG)
- `image` dependency for PNG image rendering
- `position remove` command to clear a fixed GPS position, allowing the device to fall back to GPS hardware if available
- `device get-ringtone` command to display the stored ringtone on the device, with configurable `--timeout`
- `device reboot-ota` command to reboot into OTA firmware update mode (ESP32 devices), supporting remote nodes via `--dest`/`--to`
- `device enter-dfu` command to enter DFU (Device Firmware Update) mode on NRF52 devices
- `device factory-reset-device` command for a full factory reset including BLE bonds (distinct from `factory-reset`, which preserves BLE bonds)
- `send --ack` flag to wait for delivery confirmation (ACK) before returning, with configurable `--timeout` (default 30s)
- `send --private` flag to send messages via `PRIVATE_APP` port (port 256) instead of the standard text message port
- `channel qr --all` flag to generate individual QR codes for each active channel
- `config begin-edit` command to signal the device to begin a batch of configuration changes
- `config commit-edit` command to signal the device to commit and apply a batch of configuration changes
- `config set-modem-preset` command to set the LoRa modem preset directly (LongFast, LongSlow, VeryLongSlow, MediumSlow, MediumFast, ShortSlow, ShortFast, LongModerate, ShortTurbo)
- `config ch-add-url` command to add channels from a `meshtastic://` URL without replacing existing channels
- `reply` command to auto-reply to incoming text messages with signal information (SNR, RSSI, hops)
- `gpio write` command to set GPIO pins on a remote node via `RemoteHardwareApp`
- `gpio read` command to read GPIO pin states from a remote node with configurable timeout
- `gpio watch` command to monitor GPIO pin changes on a remote node in real time
- `support` command to display diagnostic information including CLI version, firmware, hardware, region, channels, and known nodes
- `nodes --fields` flag to select which columns to display, with support for extended fields (`hw_model`, `role`, `position`)
- `--ble <name|mac>` global connection option for BLE connectivity to Meshtastic devices (requires `--features ble` build flag)
- `--ble-scan` global flag to scan for nearby BLE Meshtastic devices
- `--no-nodes` global flag to skip node collection during connection for faster startup on commands that do not require the node database
- `position set --flags` option to set position broadcast field flags when configuring a fixed position; now accepts comma-separated flag names (ALTITUDE, ALTITUDE_MSL, GEOIDAL_SEPARATION, DOP, HVDOP, SATINVIEW, SEQ_NO, TIMESTAMP, HEADING, SPEED) in addition to numeric bitmask
- `ble` feature flag in `Cargo.toml` with conditional compilation for BLE support via `meshtastic/bluetooth-le`
- `request telemetry --type` flag to select telemetry variant: device (default), environment, air-quality, power, local-stats, health, host
- Full telemetry display for all 7 variants: AirQualityMetrics (PM1.0/2.5/10.0, CO2, VOC, NOx), LocalStats (uptime, utilization, packets), HealthMetrics (heart rate, SpO2), HostMetrics (memory, disk, load average), extended PowerMetrics (ch2/ch3 voltage/current)
- `node set-unmessageable` command to mark the local node as unmessageable or restore it as messageable

## [0.2.0] - 2026-02-20

### Added

- `send` command to send text messages to the mesh network
- Broadcast mode: send to all nodes (default when `--dest` and `--to` are omitted)
- Targeted mode by ID: send to a specific node via `--dest !abcd1234` (hex node ID)
- Targeted mode by name: send to a node via `--to Pedro` (case-insensitive name lookup)
- Name resolution with disambiguation: if multiple nodes match a name, displays the list with hex IDs
- Channel selection via `--channel` flag (0-7, default: 0)
- Node ID parsing with `!` prefix support and hex validation
- `NodeDb::find_by_name()` for case-insensitive node name search
- Confirmation output with colored checkmark on successful send

### Changed

- `create_command()` factory now returns `Result` to handle argument validation errors at command creation time

## [0.1.0] - 2026-02-20

### Added

- Initial project setup with Rust edition 2021 and Tokio async runtime
- TCP connection support via `--host` and `--port` flags (default: 127.0.0.1:4403)
- Serial connection support via `--serial` flag for physical Meshtastic devices
- `nodes` command to list all mesh nodes with ID, name, battery level, SNR, hop count, and last heard timestamp
- Colored terminal output with local node highlighted for quick identification
- Docker simulator support with `config.yaml` for local development and testing
- Command architecture using the Strategy pattern, designed for extensibility as new commands are added
- Comprehensive error handling using `thiserror` for typed errors and `anyhow` for top-level propagation
- `cli.rs` module with Clap-based argument and subcommand definitions
- `connection.rs` module abstracting TCP and serial transport behind a common interface
- `error.rs` module with project-wide error types
- `commands/` module directory with one file per command following single-responsibility principles

[Unreleased]: https://github.com/matutetandil/meshtastic-cli/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/matutetandil/meshtastic-cli/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/matutetandil/meshtastic-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/matutetandil/meshtastic-cli/releases/tag/v0.1.0
