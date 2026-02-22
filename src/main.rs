mod cli;
mod commands;
mod config_file;
mod connection;
mod error;
mod node_db;
mod router;

use clap::Parser;

use cli::{Cli, Commands};
use commands::{create_command, handle_config_file, CommandContext};
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
