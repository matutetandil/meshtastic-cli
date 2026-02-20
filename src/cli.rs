use clap::{Args, Parser, Subcommand, ValueEnum};

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

    /// Get or set device configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

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

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Display current configuration
    Get {
        /// Config section name (e.g. device, lora, mqtt). Omit to show all.
        section: Option<ConfigSection>,
    },
    /// Set a configuration value (causes device reboot)
    Set {
        /// Config key in section.field format (e.g. lora.region, mqtt.enabled)
        key: String,
        /// New value to set
        value: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ConfigSection {
    Device,
    Position,
    Power,
    Network,
    Display,
    Lora,
    Bluetooth,
    Security,
    Mqtt,
    Serial,
    ExternalNotification,
    StoreForward,
    RangeTest,
    Telemetry,
    CannedMessage,
    Audio,
    RemoteHardware,
    NeighborInfo,
    AmbientLighting,
    DetectionSensor,
    Paxcounter,
}
