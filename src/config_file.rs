use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::cli::ConnectionArgs;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub serial: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ble: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json: Option<bool>,
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("meshtastic-cli")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn load() -> AppConfig {
    let path = config_path();
    if !path.exists() {
        return AppConfig::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save(config: &AppConfig) -> anyhow::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create config directory: {}", dir.display()))?;

    let path = config_path();
    let contents = toml::to_string_pretty(config).context("Failed to serialize config to TOML")?;

    std::fs::write(&path, contents)
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;

    Ok(())
}

pub fn merge_with_cli(config: &AppConfig, cli: &mut ConnectionArgs) {
    // CLI flags take precedence over config file values.
    // Only apply config values where the CLI has its default.

    if cli.host == "127.0.0.1" {
        if let Some(ref host) = config.host {
            cli.host = host.clone();
        }
    }

    if cli.port == 4403 {
        if let Some(port) = config.port {
            cli.port = port;
        }
    }

    if cli.serial.is_none() {
        if let Some(ref serial) = config.serial {
            cli.serial = Some(serial.clone());
        }
    }

    if cli.ble.is_none() {
        if let Some(ref ble) = config.ble {
            cli.ble = Some(ble.clone());
        }
    }

    if !cli.json {
        if let Some(true) = config.json {
            cli.json = true;
        }
    }
}
