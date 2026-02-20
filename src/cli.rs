use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "meshtastic-cli")]
#[command(about = "CLI tool for interacting with Meshtastic mesh networking devices")]
#[command(version)]
pub struct Cli {
    #[command(flatten)]
    pub connection: ConnectionArgs,

    #[command(subcommand)]
    pub command: Option<Commands>,
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

    /// BLE device name or MAC address (requires --features ble)
    #[arg(long)]
    pub ble: Option<String>,

    /// Scan for available BLE Meshtastic devices and exit
    #[arg(long)]
    pub ble_scan: bool,

    /// Skip collecting nodes during connection (faster startup)
    #[arg(long)]
    pub no_nodes: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all nodes in the mesh network
    Nodes {
        /// Comma-separated list of fields to display (e.g. id,name,battery,snr,hops,last_heard,hw_model,role,position)
        #[arg(long)]
        fields: Option<String>,
    },

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

        /// Wait for delivery acknowledgment (requires --dest or --to)
        #[arg(long)]
        ack: bool,

        /// Timeout in seconds when waiting for ACK
        #[arg(long, default_value_t = 30)]
        timeout: u64,

        /// Send via PRIVATE_APP port instead of TEXT_MESSAGE_APP
        #[arg(long)]
        private: bool,
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

    /// Node management (set-owner, remove)
    Node {
        #[command(subcommand)]
        action: NodeAction,
    },

    /// GPS position management (get, set)
    Position {
        #[command(subcommand)]
        action: PositionAction,
    },

    /// Request data from remote nodes
    Request {
        #[command(subcommand)]
        action: RequestAction,
    },

    /// Device management (reboot, shutdown, factory-reset)
    Device {
        #[command(subcommand)]
        action: DeviceAction,
    },

    /// Manage channels (add, delete, set properties)
    Channel {
        #[command(subcommand)]
        action: ChannelAction,
    },

    /// Auto-reply to incoming messages with signal info (SNR, RSSI, hops)
    Reply,

    /// Remote GPIO operations
    Gpio {
        #[command(subcommand)]
        action: GpioAction,
    },

    /// Print diagnostic info for support/troubleshooting
    Support,

    /// Trace route to a node, showing each hop with SNR
    Traceroute {
        /// Destination node ID in hex (e.g. 04e1c43b or '!04e1c43b')
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Destination node name (e.g. Pedro). Searches known nodes by name.
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,

        /// Timeout in seconds to wait for traceroute response
        #[arg(long, default_value_t = 60)]
        timeout: u64,
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
    /// Export full device configuration to YAML
    Export {
        /// Output file path (prints to stdout if omitted)
        #[arg(long)]
        file: Option<String>,
    },
    /// Import device configuration from a YAML file
    Import {
        /// YAML configuration file to import
        file: String,
    },
    /// Configure licensed Ham radio mode
    SetHam {
        /// Amateur radio call sign (e.g. KD2ABC)
        call_sign: String,

        /// Short name (up to 5 characters). Defaults to first 4 chars of call sign.
        #[arg(long)]
        short: Option<String>,

        /// Transmit power in dBm
        #[arg(long)]
        tx_power: Option<i32>,

        /// LoRa frequency in MHz
        #[arg(long)]
        frequency: Option<f32>,
    },
    /// Apply channels and LoRa config from a meshtastic:// URL
    SetUrl {
        /// Meshtastic URL (e.g. https://meshtastic.org/e/#... or meshtastic://...)
        url: String,
    },
    /// Begin a batch editing session (changes are queued until commit)
    BeginEdit,
    /// Commit queued configuration changes from a batch editing session
    CommitEdit,
    /// Set the LoRa modem preset
    SetModemPreset {
        /// Modem preset to apply
        preset: ModemPresetArg,
    },
    /// Add channels from a meshtastic:// URL without replacing existing ones
    ChAddUrl {
        /// Meshtastic URL (e.g. https://meshtastic.org/e/#... or meshtastic://...)
        url: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum NodeAction {
    /// Set the device owner name
    SetOwner {
        /// Long name for the device (up to 40 characters)
        name: String,

        /// Short name (up to 5 characters). Auto-generated from long name if omitted.
        #[arg(long)]
        short: Option<String>,
    },
    /// Remove a node from the local NodeDB
    Remove {
        /// Node ID in hex (e.g. 04e1c43b)
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Node name to remove
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,
    },
    /// Mark a node as favorite
    SetFavorite {
        /// Node ID in hex (e.g. 04e1c43b)
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Node name to mark as favorite
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,
    },
    /// Remove a node from favorites
    RemoveFavorite {
        /// Node ID in hex (e.g. 04e1c43b)
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Node name to remove from favorites
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,
    },
    /// Mark a node as ignored
    SetIgnored {
        /// Node ID in hex (e.g. 04e1c43b)
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Node name to ignore
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,
    },
    /// Set the node as unmessageable (prevents others from messaging it)
    SetUnmessageable {
        /// true to mark as unmessageable, false to mark as messageable
        #[arg(action = clap::ArgAction::Set, value_parser = clap::builder::BoolishValueParser::new(), default_value_t = true)]
        value: bool,
    },
    /// Remove a node from the ignored list
    RemoveIgnored {
        /// Node ID in hex (e.g. 04e1c43b)
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Node name to remove from ignored list
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum DeviceAction {
    /// Reboot the device (local or remote)
    Reboot {
        /// Target node ID in hex (e.g. 04e1c43b). Omit to reboot local device.
        #[arg(long, conflicts_with = "to")]
        dest: Option<String>,

        /// Target node name. Omit to reboot local device.
        #[arg(long, conflicts_with = "dest")]
        to: Option<String>,

        /// Delay in seconds before rebooting
        #[arg(long, default_value_t = 5)]
        delay: i32,
    },
    /// Factory reset configuration (restores defaults, preserves BLE bonds)
    FactoryReset,
    /// Clear the node database
    ResetNodedb,
    /// Shut down the device (local or remote)
    Shutdown {
        /// Target node ID in hex (e.g. 04e1c43b). Omit to shut down local device.
        #[arg(long, conflicts_with = "to")]
        dest: Option<String>,

        /// Target node name. Omit to shut down local device.
        #[arg(long, conflicts_with = "dest")]
        to: Option<String>,

        /// Delay in seconds before shutting down
        #[arg(long, default_value_t = 5)]
        delay: i32,
    },
    /// Set the device clock (uses current system time if no timestamp given)
    SetTime {
        /// Unix timestamp to set. Uses current system time if omitted.
        time: Option<u32>,
    },
    /// Set canned messages (separated by '|')
    SetCannedMessage {
        /// Messages separated by '|' (e.g. "Yes|No|Help|SOS")
        message: String,
    },
    /// Display the configured canned messages
    GetCannedMessage {
        /// Timeout in seconds to wait for response
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },
    /// Set the device ringtone (RTTTL format)
    SetRingtone {
        /// Ringtone in RTTTL format (e.g. "ring:d=4,o=5,b=120:c,e,g")
        ringtone: String,
    },
    /// Display the configured ringtone
    GetRingtone {
        /// Timeout in seconds to wait for response
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },
    /// Reboot into OTA update mode (ESP32 devices)
    RebootOta {
        /// Target node ID in hex (e.g. 04e1c43b). Omit to reboot local device.
        #[arg(long, conflicts_with = "to")]
        dest: Option<String>,

        /// Target node name. Omit to reboot local device.
        #[arg(long, conflicts_with = "dest")]
        to: Option<String>,

        /// Delay in seconds before rebooting
        #[arg(long, default_value_t = 5)]
        delay: i32,
    },
    /// Enter DFU mode for firmware update (NRF52 devices)
    EnterDfu,
    /// Full factory reset (wipes everything including BLE bonds)
    FactoryResetDevice,
}

#[derive(Subcommand, Debug)]
pub enum PositionAction {
    /// Display current GPS position
    Get,
    /// Set a fixed GPS position
    Set {
        /// Latitude in decimal degrees (e.g. 40.4168)
        lat: f64,
        /// Longitude in decimal degrees (e.g. -3.7038)
        lon: f64,
        /// Altitude in meters above sea level
        #[arg(default_value_t = 0)]
        alt: i32,
        /// Position broadcast field flags. Accepts a numeric bitmask or comma-separated names:
        /// ALTITUDE, ALTITUDE_MSL, GEOIDAL_SEPARATION, DOP, HVDOP, SATINVIEW, SEQ_NO, TIMESTAMP, HEADING, SPEED
        #[arg(long)]
        flags: Option<String>,
    },
    /// Remove the fixed GPS position (re-enables GPS)
    Remove,
}

#[derive(Subcommand, Debug)]
pub enum RequestAction {
    /// Request telemetry from a remote node
    Telemetry {
        /// Target node ID in hex
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Target node name
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,

        /// Timeout in seconds
        #[arg(long, default_value_t = 30)]
        timeout: u64,

        /// Type of telemetry to request
        #[arg(long, value_enum, default_value_t = TelemetryTypeArg::Device)]
        r#type: TelemetryTypeArg,
    },
    /// Request position from a remote node
    Position {
        /// Target node ID in hex
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Target node name
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,

        /// Timeout in seconds
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },
    /// Request device metadata from a remote node
    Metadata {
        /// Target node ID in hex
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Target node name
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,

        /// Timeout in seconds
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },
}

#[derive(Subcommand, Debug)]
pub enum ChannelAction {
    /// Add a new secondary channel
    Add {
        /// Channel name (up to 11 characters)
        name: String,

        /// Pre-shared key: "none", "default", "random", or a hex-encoded key (32 or 64 hex chars for AES-128/256)
        #[arg(long, default_value = "default")]
        psk: String,
    },
    /// Delete a channel by index
    Del {
        /// Channel index to delete (1-7, cannot delete primary channel 0)
        index: u32,
    },
    /// Set a channel property
    Set {
        /// Channel index (0-7)
        index: u32,
        /// Field name to set (name, psk, uplink_enabled, downlink_enabled, position_precision)
        field: String,
        /// New value
        value: String,
    },
    /// List all channels (same as info, but channel-focused)
    List,
    /// Generate a QR code for sharing channels
    Qr {
        /// Output file path (.png or .svg). Prints to terminal if omitted.
        #[arg(long)]
        output: Option<String>,

        /// Generate individual QR codes for each active channel
        #[arg(long)]
        all: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum GpioAction {
    /// Write GPIO pin values on a remote node
    Write {
        /// Target node ID in hex
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Target node name
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,

        /// GPIO bitmask (decimal or 0x hex)
        #[arg(long)]
        mask: String,

        /// GPIO value (decimal or 0x hex)
        #[arg(long)]
        value: String,
    },
    /// Read GPIO pin values from a remote node
    Read {
        /// Target node ID in hex
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Target node name
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,

        /// GPIO bitmask to read (decimal or 0x hex)
        #[arg(long)]
        mask: String,

        /// Timeout in seconds
        #[arg(long, default_value_t = 30)]
        timeout: u64,
    },
    /// Watch GPIO pin changes on a remote node
    Watch {
        /// Target node ID in hex
        #[arg(long, conflicts_with = "to", required_unless_present = "to")]
        dest: Option<String>,

        /// Target node name
        #[arg(long, conflicts_with = "dest", required_unless_present = "dest")]
        to: Option<String>,

        /// GPIO bitmask to watch (decimal or 0x hex)
        #[arg(long)]
        mask: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum ModemPresetArg {
    LongFast,
    LongSlow,
    VeryLongSlow,
    MediumSlow,
    MediumFast,
    ShortSlow,
    ShortFast,
    LongModerate,
    ShortTurbo,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TelemetryTypeArg {
    Device,
    Environment,
    AirQuality,
    Power,
    LocalStats,
    Health,
    Host,
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
