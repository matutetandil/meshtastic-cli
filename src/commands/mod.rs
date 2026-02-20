mod channel;
mod config;
mod device;
mod export_import;
mod gpio;
mod info;
mod listen;
mod node;
mod nodes;
mod ping;
mod position;
mod reply;
mod request;
mod send;
mod support;
mod traceroute;

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::api::ConnectedStreamApi;
use meshtastic::packet::{PacketDestination, PacketReceiver};
use meshtastic::types::{MeshChannel, NodeId};

use crate::cli::{
    ChannelAction, Commands, ConfigAction, DeviceAction, GpioAction, ModemPresetArg, NodeAction,
    PositionAction, RequestAction, TelemetryTypeArg,
};
use crate::error::CliError;
use crate::node_db::NodeDb;
use crate::router::MeshRouter;

#[allow(dead_code)]
pub struct CommandContext {
    pub api: ConnectedStreamApi,
    pub node_db: NodeDb,
    pub packet_receiver: PacketReceiver,
    pub router: MeshRouter,
}

#[async_trait]
pub trait Command {
    async fn execute(self: Box<Self>, ctx: CommandContext) -> anyhow::Result<()>;
}

pub enum DestinationSpec {
    Broadcast,
    NodeId(u32),
    NodeName(String),
}

pub fn resolve_destination(
    spec: &DestinationSpec,
    node_db: &NodeDb,
) -> anyhow::Result<(PacketDestination, String)> {
    match spec {
        DestinationSpec::Broadcast => Ok((PacketDestination::Broadcast, "broadcast".to_string())),
        DestinationSpec::NodeId(id) => Ok((
            PacketDestination::Node(NodeId::new(*id)),
            format!("!{:08x}", id),
        )),
        DestinationSpec::NodeName(name) => {
            let matches = node_db.find_by_name(name);

            match matches.len() {
                0 => bail!(
                    "No node found with name '{}'. Use 'nodes' command to list known nodes.",
                    name
                ),
                1 => {
                    let (num, node) = &matches[0];
                    let node_name = node
                        .user
                        .as_ref()
                        .map(|u| u.long_name.as_str())
                        .unwrap_or("Unknown");
                    println!(
                        "{} Resolved '{}' to !{:08x} ({})",
                        "→".cyan(),
                        name,
                        num,
                        node_name
                    );
                    Ok((
                        PacketDestination::Node(NodeId::new(*num)),
                        format!("{} (!{:08x})", node_name, num),
                    ))
                }
                _ => {
                    let mut msg = format!(
                        "Multiple nodes found with name '{}'. Use --dest with the node ID:\n",
                        name
                    );
                    for (num, node) in &matches {
                        let node_name = node
                            .user
                            .as_ref()
                            .map(|u| u.long_name.as_str())
                            .unwrap_or("Unknown");
                        msg.push_str(&format!("  !{:08x}  {}\n", num, node_name));
                    }
                    bail!("{}", msg.trim_end())
                }
            }
        }
    }
}

fn parse_dest_spec(
    dest: &Option<String>,
    to: &Option<String>,
) -> Result<DestinationSpec, CliError> {
    match (dest, to) {
        (Some(hex_str), None) => {
            let stripped = hex_str.strip_prefix('!').unwrap_or(hex_str);
            let node_num = u32::from_str_radix(stripped, 16).map_err(|_| {
                CliError::InvalidArgument(format!(
                    "Invalid node ID '{}'. Expected hex format like !abcd1234",
                    hex_str
                ))
            })?;
            Ok(DestinationSpec::NodeId(node_num))
        }
        (None, Some(name)) => Ok(DestinationSpec::NodeName(name.clone())),
        _ => Ok(DestinationSpec::Broadcast),
    }
}

pub fn create_command(command: &Commands) -> Result<Box<dyn Command>, CliError> {
    match command {
        Commands::Nodes { fields } => {
            let parsed_fields = fields
                .as_ref()
                .map(|f| f.split(',').map(|s| s.trim().to_string()).collect());
            Ok(Box::new(nodes::NodesCommand {
                fields: parsed_fields,
            }))
        }
        Commands::Listen => Ok(Box::new(listen::ListenCommand)),
        Commands::Reply => Ok(Box::new(reply::ReplyCommand)),
        Commands::Support => Ok(Box::new(support::SupportCommand)),
        Commands::Gpio { action } => match action {
            GpioAction::Write {
                dest,
                to,
                mask,
                value,
            } => {
                let destination = parse_dest_spec(dest, to)?;
                let mask_val = gpio::parse_bitmask(mask)
                    .map_err(|e| CliError::InvalidArgument(e.to_string()))?;
                let gpio_val = gpio::parse_bitmask(value)
                    .map_err(|e| CliError::InvalidArgument(e.to_string()))?;
                Ok(Box::new(gpio::GpioWriteCommand {
                    destination,
                    mask: mask_val,
                    value: gpio_val,
                }))
            }
            GpioAction::Read {
                dest,
                to,
                mask,
                timeout,
            } => {
                let destination = parse_dest_spec(dest, to)?;
                let mask_val = gpio::parse_bitmask(mask)
                    .map_err(|e| CliError::InvalidArgument(e.to_string()))?;
                Ok(Box::new(gpio::GpioReadCommand {
                    destination,
                    mask: mask_val,
                    timeout_secs: *timeout,
                }))
            }
            GpioAction::Watch { dest, to, mask } => {
                let destination = parse_dest_spec(dest, to)?;
                let mask_val = gpio::parse_bitmask(mask)
                    .map_err(|e| CliError::InvalidArgument(e.to_string()))?;
                Ok(Box::new(gpio::GpioWatchCommand {
                    destination,
                    mask: mask_val,
                }))
            }
        },
        Commands::Info => Ok(Box::new(info::InfoCommand)),
        Commands::Send {
            message,
            dest,
            to,
            channel,
            ack,
            timeout,
            private,
        } => {
            let destination = parse_dest_spec(dest, to)?;
            let mesh_channel = MeshChannel::new(*channel)
                .map_err(|e| CliError::InvalidArgument(format!("Invalid channel index: {}", e)))?;

            Ok(Box::new(send::SendCommand {
                message: message.clone(),
                destination,
                channel: mesh_channel,
                wait_ack: *ack,
                timeout_secs: *timeout,
                private: *private,
            }))
        }
        Commands::Ping { dest, to, timeout } => {
            let destination = parse_dest_spec(dest, to)?;
            Ok(Box::new(ping::PingCommand {
                destination,
                timeout_secs: *timeout,
            }))
        }
        Commands::Traceroute { dest, to, timeout } => {
            let destination = parse_dest_spec(dest, to)?;
            Ok(Box::new(traceroute::TracerouteCommand {
                destination,
                timeout_secs: *timeout,
            }))
        }
        Commands::Node { action } => match action {
            NodeAction::SetOwner { name, short } => Ok(Box::new(node::SetOwnerCommand {
                long_name: name.clone(),
                short_name: short.clone(),
            })),
            NodeAction::Remove { dest, to } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(node::RemoveNodeCommand { destination }))
            }
            NodeAction::SetFavorite { dest, to } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(node::SetFavoriteCommand { destination }))
            }
            NodeAction::RemoveFavorite { dest, to } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(node::RemoveFavoriteCommand { destination }))
            }
            NodeAction::SetIgnored { dest, to } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(node::SetIgnoredCommand { destination }))
            }
            NodeAction::SetUnmessageable { value } => {
                Ok(Box::new(node::SetUnmessageableCommand { value: *value }))
            }
            NodeAction::RemoveIgnored { dest, to } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(node::RemoveIgnoredCommand { destination }))
            }
        },
        Commands::Position { action } => match action {
            PositionAction::Get => Ok(Box::new(position::PositionGetCommand)),
            PositionAction::Set {
                lat,
                lon,
                alt,
                flags,
            } => {
                let parsed_flags = flags
                    .as_ref()
                    .map(|f| position::parse_position_flags(f))
                    .transpose()
                    .map_err(|e| CliError::InvalidArgument(e.to_string()))?;
                Ok(Box::new(position::PositionSetCommand {
                    latitude: *lat,
                    longitude: *lon,
                    altitude: *alt,
                    flags: parsed_flags,
                }))
            }
            PositionAction::Remove => Ok(Box::new(position::PositionRemoveCommand)),
        },
        Commands::Request { action } => match action {
            RequestAction::Telemetry {
                dest,
                to,
                timeout,
                r#type,
            } => {
                let destination = parse_dest_spec(dest, to)?;
                let telem_type = match r#type {
                    TelemetryTypeArg::Device => request::TelemetryType::Device,
                    TelemetryTypeArg::Environment => request::TelemetryType::Environment,
                    TelemetryTypeArg::AirQuality => request::TelemetryType::AirQuality,
                    TelemetryTypeArg::Power => request::TelemetryType::Power,
                    TelemetryTypeArg::LocalStats => request::TelemetryType::LocalStats,
                    TelemetryTypeArg::Health => request::TelemetryType::Health,
                    TelemetryTypeArg::Host => request::TelemetryType::Host,
                };
                Ok(Box::new(request::RequestTelemetryCommand {
                    destination,
                    timeout_secs: *timeout,
                    telemetry_type: telem_type,
                }))
            }
            RequestAction::Position { dest, to, timeout } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(request::RequestPositionCommand {
                    destination,
                    timeout_secs: *timeout,
                }))
            }
            RequestAction::Metadata { dest, to, timeout } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(request::RequestMetadataCommand {
                    destination,
                    timeout_secs: *timeout,
                }))
            }
        },
        Commands::Device { action } => match action {
            DeviceAction::Reboot { dest, to, delay } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(device::RebootCommand {
                    destination,
                    delay_secs: *delay,
                }))
            }
            DeviceAction::FactoryReset => Ok(Box::new(device::FactoryResetCommand)),
            DeviceAction::ResetNodedb => Ok(Box::new(device::ResetNodeDbCommand)),
            DeviceAction::Shutdown { dest, to, delay } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(device::ShutdownCommand {
                    destination,
                    delay_secs: *delay,
                }))
            }
            DeviceAction::SetTime { time } => Ok(Box::new(device::SetTimeCommand { time: *time })),
            DeviceAction::SetCannedMessage { message } => {
                Ok(Box::new(device::SetCannedMessageCommand {
                    message: message.clone(),
                }))
            }
            DeviceAction::GetCannedMessage { timeout } => {
                Ok(Box::new(device::GetCannedMessageCommand {
                    timeout_secs: *timeout,
                }))
            }
            DeviceAction::SetRingtone { ringtone } => Ok(Box::new(device::SetRingtoneCommand {
                ringtone: ringtone.clone(),
            })),
            DeviceAction::GetRingtone { timeout } => Ok(Box::new(device::GetRingtoneCommand {
                timeout_secs: *timeout,
            })),
            DeviceAction::RebootOta { dest, to, delay } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(device::RebootOtaCommand {
                    destination,
                    delay_secs: *delay,
                }))
            }
            DeviceAction::EnterDfu => Ok(Box::new(device::EnterDfuCommand)),
            DeviceAction::FactoryResetDevice => Ok(Box::new(device::FactoryResetDeviceCommand)),
        },
        Commands::Channel { action } => match action {
            ChannelAction::List => Ok(Box::new(channel::ChannelListCommand)),
            ChannelAction::Add { name, psk } => Ok(Box::new(channel::ChannelAddCommand {
                name: name.clone(),
                psk: psk.clone(),
            })),
            ChannelAction::Del { index } => {
                Ok(Box::new(channel::ChannelDelCommand { index: *index }))
            }
            ChannelAction::Set {
                index,
                field,
                value,
            } => Ok(Box::new(channel::ChannelSetCommand {
                index: *index,
                field: field.clone(),
                value: value.clone(),
            })),
            ChannelAction::Qr { output, all } => Ok(Box::new(channel::ChannelQrCommand {
                output: output.clone(),
                all: *all,
            })),
        },
        Commands::Config { action } => match action {
            ConfigAction::Get { section } => Ok(Box::new(config::ConfigGetCommand {
                section: section.clone(),
            })),
            ConfigAction::Set { key, value } => Ok(Box::new(config::ConfigSetCommand {
                key: key.clone(),
                value: value.clone(),
            })),
            ConfigAction::Export { file } => Ok(Box::new(export_import::ExportConfigCommand {
                file: file.as_ref().map(std::path::PathBuf::from),
            })),
            ConfigAction::Import { file } => Ok(Box::new(export_import::ImportConfigCommand {
                file: std::path::PathBuf::from(file),
            })),
            ConfigAction::SetHam {
                call_sign,
                short,
                tx_power,
                frequency,
            } => Ok(Box::new(config::SetHamCommand {
                call_sign: call_sign.clone(),
                short_name: short.clone(),
                tx_power: *tx_power,
                frequency: *frequency,
            })),
            ConfigAction::SetUrl { url } => {
                Ok(Box::new(config::SetUrlCommand { url: url.clone() }))
            }
            ConfigAction::BeginEdit => Ok(Box::new(config::BeginEditCommand)),
            ConfigAction::CommitEdit => Ok(Box::new(config::CommitEditCommand)),
            ConfigAction::SetModemPreset { preset } => {
                let preset_i32 = match preset {
                    ModemPresetArg::LongFast => 0,
                    ModemPresetArg::LongSlow => 1,
                    ModemPresetArg::VeryLongSlow => 2,
                    ModemPresetArg::MediumSlow => 3,
                    ModemPresetArg::MediumFast => 4,
                    ModemPresetArg::ShortSlow => 5,
                    ModemPresetArg::ShortFast => 6,
                    ModemPresetArg::LongModerate => 7,
                    ModemPresetArg::ShortTurbo => 8,
                };
                Ok(Box::new(config::SetModemPresetCommand {
                    preset: preset_i32,
                }))
            }
            ConfigAction::ChAddUrl { url } => {
                Ok(Box::new(config::ChAddUrlCommand { url: url.clone() }))
            }
        },
    }
}
