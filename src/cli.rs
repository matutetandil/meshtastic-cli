use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "meshtastic-cli")]
#[command(about = "CLI tool for interacting with Meshtastic mesh networking devices")]
#[command(version)]
pub struct Cli {
    #[command(flatten)]
    pub connection: ConnectionArgs,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args, Debug)]
pub struct ConnectionArgs {
    /// Host address for TCP connection
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    /// Port for TCP connection
    #[arg(long, default_value_t = 4403)]
    pub port: u16,

    /// Serial port path (overrides TCP connection)
    #[arg(long)]
    pub serial: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all nodes in the mesh network
    Nodes,
}
