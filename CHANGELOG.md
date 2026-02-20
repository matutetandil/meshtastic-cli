# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/your-org/meshtastic-cli/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/your-org/meshtastic-cli/releases/tag/v0.1.0
