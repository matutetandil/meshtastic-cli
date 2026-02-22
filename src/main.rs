mod cli;
mod commands;
mod config_file;
mod connection;
mod error;
mod node_db;
mod router;

use clap::Parser;

use cli::{Cli, Commands, ConfigFileAction};
use commands::{create_command, CommandContext};
use router::MeshRouter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut cli = Cli::parse();

    // Load persistent config and merge with CLI defaults
    let app_config = config_file::load();
    config_file::merge_with_cli(&app_config, &mut cli.connection);

    if cli.connection.ble_scan {
        connection::scan_ble_devices().await?;
        return Ok(());
    }

    let Some(ref cmd) = cli.command else {
        eprintln!("No command specified. Run with --help for usage.");
        std::process::exit(1);
    };

    // Handle commands that don't need a device connection
    match cmd {
        Commands::Completions { shell } => {
            Cli::generate_completions(*shell);
            return Ok(());
        }
        Commands::ConfigFile { action } => {
            handle_config_file(action, cli.connection.json)?;
            return Ok(());
        }
        _ => {}
    }

    let conn = connection::establish(&cli.connection).await?;
    let router = MeshRouter::new(conn.node_db.my_node_num());

    let mut ctx = CommandContext {
        api: conn.api,
        node_db: conn.node_db,
        packet_receiver: conn.packet_receiver,
        router,
        json: cli.connection.json,
    };

    let command = create_command(cmd)?;
    command.execute(&mut ctx).await?;

    Ok(())
}

fn handle_config_file(action: &ConfigFileAction, json: bool) -> anyhow::Result<()> {
    use colored::Colorize;

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
                    config.json = Some(parse_bool_value(value).ok_or_else(|| {
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

fn parse_bool_value(s: &str) -> Option<bool> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}
