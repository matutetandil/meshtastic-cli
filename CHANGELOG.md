# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
- `qrcode` dependency for terminal QR code generation

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

[Unreleased]: https://github.com/your-org/meshtastic-cli/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/your-org/meshtastic-cli/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/your-org/meshtastic-cli/releases/tag/v0.1.0
