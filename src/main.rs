mod cli;
mod commands;
mod connection;
mod error;
mod node_db;
mod router;

use clap::Parser;

use cli::Cli;
use commands::{create_command, CommandContext};
use router::MeshRouter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    let conn = connection::establish(&cli.connection).await?;
    let router = MeshRouter::new(conn.node_db.my_node_num());

    let ctx = CommandContext {
        api: conn.api,
        node_db: conn.node_db,
        packet_receiver: conn.packet_receiver,
        router,
    };

    let command = create_command(&cli.command);
    command.execute(ctx).await?;

    Ok(())
}
