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

    /// Send a text message to the mesh network
    Send {
        /// The message text to send
        message: String,

        /// Destination node ID in hex (e.g. 04e1c43b or '!04e1c43b'). Omit to broadcast.
        #[arg(long, conflicts_with = "to")]
        dest: Option<String>,

        /// Destination node name (e.g. Pedro). Searches known nodes by name.
        #[arg(long, conflicts_with = "dest")]
        to: Option<String>,

        /// Channel index (0-7)
        #[arg(long, default_value_t = 0)]
        channel: u32,
    },

    /// Stream incoming packets from the mesh network in real time
    Listen,

    /// Show local node and device information
    Info,

    /// Ping a node and wait for ACK to measure round-trip time
    Ping {
        /// Destination node ID in hex (e.g. 04e1c43b or '!04e1c43b')
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Destination node name (e.g. Pedro). Searches known nodes by name.
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,

        /// Timeout in seconds to wait for ACK
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },
}
