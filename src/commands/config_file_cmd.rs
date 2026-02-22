use colored::Colorize;

use crate::cli::ConfigFileAction;
use crate::config_file;

use super::parsers::parse_bool;

pub fn handle_config_file(action: &ConfigFileAction, json: bool) -> anyhow::Result<()> {
    match action {
        ConfigFileAction::Show => {
            let config = config_file::load();
            if json {
                println!("{}", serde_json::to_string_pretty(&config)?);
                return Ok(());
            }

            let path = config_file::config_path();
            println!("{}", "Config File".bold().underline());
            println!("  {:<12} {}", "path:".dimmed(), path.display());
            println!();

            if !path.exists() {
                println!("  {}", "(no config file found — using defaults)".dimmed());
                return Ok(());
            }

            let has_any = config.host.is_some()
                || config.port.is_some()
                || config.serial.is_some()
                || config.ble.is_some()
                || config.json.is_some();

            if !has_any {
                println!("  {}", "(config file is empty)".dimmed());
                return Ok(());
            }

            if let Some(ref host) = config.host {
                println!("  {:<12} {}", "host:".dimmed(), host);
            }
            if let Some(port) = config.port {
                println!("  {:<12} {}", "port:".dimmed(), port);
            }
            if let Some(ref serial) = config.serial {
                println!("  {:<12} {}", "serial:".dimmed(), serial);
            }
            if let Some(ref ble) = config.ble {
                println!("  {:<12} {}", "ble:".dimmed(), ble);
            }
            if let Some(json_val) = config.json {
                println!("  {:<12} {}", "json:".dimmed(), json_val);
            }
        }
        ConfigFileAction::Set { key, value } => {
            let mut config = config_file::load();

            match key.as_str() {
                "host" => config.host = Some(value.clone()),
                "port" => {
                    config.port = Some(value.parse().map_err(|_| {
                        anyhow::anyhow!("Invalid port '{}'. Expected a number 1-65535.", value)
                    })?);
                }
                "serial" => config.serial = Some(value.clone()),
                "ble" => config.ble = Some(value.clone()),
                "json" => {
                    config.json = Some(parse_bool(value).map_err(|_| {
                        anyhow::anyhow!("Invalid value '{}' for json. Expected true/false.", value)
                    })?);
                }
                other => {
                    anyhow::bail!(
                        "Unknown config key '{}'. Valid keys: host, port, serial, ble, json",
                        other
                    );
                }
            }

            config_file::save(&config)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&config)?);
            } else {
                println!("{} Set {} = {}", "ok".green(), key.bold(), value);
            }
        }
        ConfigFileAction::Unset { key } => {
            let mut config = config_file::load();

            match key.as_str() {
                "host" => config.host = None,
                "port" => config.port = None,
                "serial" => config.serial = None,
                "ble" => config.ble = None,
                "json" => config.json = None,
                other => {
                    anyhow::bail!(
                        "Unknown config key '{}'. Valid keys: host, port, serial, ble, json",
                        other
                    );
                }
            }

            config_file::save(&config)?;

            if json {
                println!("{}", serde_json::to_string_pretty(&config)?);
            } else {
                println!("{} Unset {}", "ok".green(), key.bold());
            }
        }
        ConfigFileAction::Path => {
            let path = config_file::config_path();
            if json {
                println!(
                    "{}",
                    serde_json::json!({ "path": path.display().to_string() })
                );
            } else {
                println!("{}", path.display());
            }
        }
    }

    Ok(())
}
