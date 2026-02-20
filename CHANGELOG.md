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
