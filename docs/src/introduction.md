# Introduction

> A Rust CLI tool for interacting with Meshtastic mesh networking devices over TCP, serial, or BLE connections.

## What it does

`mttctl` provides a command-line interface to Meshtastic devices, allowing you to list nodes, send messages, monitor incoming packets, query device info, ping specific nodes, manage channels, control GPIO pins, and more — all from a terminal.

## Why it exists

The Meshtastic ecosystem lacks a robust, composable CLI tool built in Rust. This project aims to fill that gap as an open-source contribution, leveraging the official [`meshtastic` Rust crate](https://github.com/meshtastic/rust) to interact with real hardware and local simulators alike.

## Who it's for

Developers and operators working with Meshtastic mesh networks who want scriptable, terminal-native access to device data without a GUI.

## Key Design Decisions

- **Strategy pattern for commands**: each command is an independent module implementing a shared trait, making it trivial to add new commands without touching existing ones.
- **SOLID principles throughout**: single responsibility per module, open/closed for command extension, dependency inversion via connection abstraction.
- **Thin `main.rs`**: only parses CLI arguments and dispatches to the appropriate command — no business logic lives there.
- **Async-first**: all I/O uses Tokio, matching the async model of the underlying `meshtastic` crate.
- **Optional BLE support**: compiled in via `--features ble` to avoid requiring Bluetooth dependencies in environments that do not need them.
