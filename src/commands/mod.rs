mod admin;
mod channel;
mod config;
mod config_file_cmd;
mod destination;
mod device;
mod export_import;
mod gpio;
mod info;
mod listen;
mod mqtt_bridge;
mod node;
mod nodes;
pub(crate) mod parsers;
mod ping;
mod position;
mod reply;
mod request;
mod send;
mod shell;
mod support;
mod traceroute;
mod watch;
mod waypoint;

use async_trait::async_trait;
use meshtastic::api::ConnectedStreamApi;
use meshtastic::packet::PacketReceiver;
use meshtastic::types::MeshChannel;

use crate::cli::{
    ChannelAction, Commands, ConfigAction, DeviceAction, GpioAction, MqttAction, NodeAction,
    PositionAction, RequestAction, WaypointAction,
};
use crate::error::CliError;
use crate::node_db::NodeDb;
use crate::router::MeshRouter;

pub use config_file_cmd::handle_config_file;
pub use destination::{parse_dest_spec, resolve_destination, DestinationSpec};

#[allow(dead_code)]
pub struct CommandContext {
    pub api: ConnectedStreamApi,
    pub node_db: NodeDb,
    pub packet_receiver: PacketReceiver,
    pub router: MeshRouter,
}

#[async_trait]
pub trait Command {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()>;
}

pub fn create_command(command: &Commands, json: bool) -> Result<Box<dyn Command + Send>, CliError> {
    match command {
        Commands::Nodes { fields } => {
            let parsed_fields = fields
                .as_ref()
                .map(|f| f.split(',').map(|s| s.trim().to_string()).collect());
            Ok(Box::new(nodes::NodesCommand {
                fields: parsed_fields,
                json,
            }))
        }
        Commands::Listen { log } => Ok(Box::new(listen::ListenCommand {
            log_path: log.as_ref().map(std::path::PathBuf::from),
            json,
        })),
        Commands::Reply => Ok(Box::new(reply::ReplyCommand { json })),
        Commands::Shell => Ok(Box::new(shell::ShellCommand)),
        Commands::Support => Ok(Box::new(support::SupportCommand { json })),
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
                    json,
                }))
            }
            GpioAction::Watch { dest, to, mask } => {
                let destination = parse_dest_spec(dest, to)?;
                let mask_val = gpio::parse_bitmask(mask)
                    .map_err(|e| CliError::InvalidArgument(e.to_string()))?;
                Ok(Box::new(gpio::GpioWatchCommand {
                    destination,
                    mask: mask_val,
                    json,
                }))
            }
        },
        Commands::Mqtt { action } => match action {
            MqttAction::Bridge {
                broker,
                port,
                topic,
                username,
                password,
            } => Ok(Box::new(mqtt_bridge::MqttBridgeCommand {
                broker: broker.clone(),
                port: *port,
                topic_prefix: topic.clone(),
                username: username.clone(),
                password: password.clone(),
                json,
            })),
        },
        Commands::Watch { interval } => Ok(Box::new(watch::WatchCommand {
            interval_secs: *interval,
            json,
        })),
        Commands::Info => Ok(Box::new(info::InfoCommand { json })),
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
                json,
            }))
        }
        Commands::Ping { dest, to, timeout } => {
            let destination = parse_dest_spec(dest, to)?;
            Ok(Box::new(ping::PingCommand {
                destination,
                timeout_secs: *timeout,
                json,
            }))
        }
        Commands::Traceroute { dest, to, timeout } => {
            let destination = parse_dest_spec(dest, to)?;
            Ok(Box::new(traceroute::TracerouteCommand {
                destination,
                timeout_secs: *timeout,
                json,
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
            PositionAction::Get => Ok(Box::new(position::PositionGetCommand { json })),
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
                Ok(Box::new(request::RequestTelemetryCommand {
                    destination,
                    timeout_secs: *timeout,
                    telemetry_type: r#type.into(),
                    json,
                }))
            }
            RequestAction::Position { dest, to, timeout } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(request::RequestPositionCommand {
                    destination,
                    timeout_secs: *timeout,
                    json,
                }))
            }
            RequestAction::Metadata { dest, to, timeout } => {
                let destination = parse_dest_spec(dest, to)?;
                Ok(Box::new(request::RequestMetadataCommand {
                    destination,
                    timeout_secs: *timeout,
                    json,
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
            ChannelAction::List => Ok(Box::new(channel::ChannelListCommand { json })),
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
                json,
            })),
        },
        Commands::Config { action } => match action {
            ConfigAction::Get { section } => Ok(Box::new(config::ConfigGetCommand {
                section: section.clone(),
                json,
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
                Ok(Box::new(config::SetModemPresetCommand {
                    preset: preset.into(),
                }))
            }
            ConfigAction::ChAddUrl { url } => {
                Ok(Box::new(config::ChAddUrlCommand { url: url.clone() }))
            }
        },
        Commands::Waypoint { action } => match action {
            WaypointAction::Send {
                lat,
                lon,
                name,
                description,
                dest,
                to,
                icon,
                expire,
                channel,
                locked,
            } => {
                let destination = parse_dest_spec(dest, to)?;
                let mesh_channel = MeshChannel::new(*channel)
                    .map_err(|e| CliError::InvalidArgument(format!("Invalid channel: {}", e)))?;
                Ok(Box::new(waypoint::WaypointSendCommand {
                    latitude: *lat,
                    longitude: *lon,
                    name: name.clone(),
                    description: description.clone(),
                    destination,
                    icon: icon.clone(),
                    expire_hours: *expire,
                    channel: mesh_channel,
                    locked: *locked,
                    json,
                }))
            }
            WaypointAction::Delete {
                id,
                dest,
                to,
                channel,
            } => {
                let destination = parse_dest_spec(dest, to)?;
                let mesh_channel = MeshChannel::new(*channel)
                    .map_err(|e| CliError::InvalidArgument(format!("Invalid channel: {}", e)))?;
                Ok(Box::new(waypoint::WaypointDeleteCommand {
                    id: *id,
                    destination,
                    channel: mesh_channel,
                    json,
                }))
            }
            WaypointAction::List { timeout } => Ok(Box::new(waypoint::WaypointListCommand {
                timeout_secs: *timeout,
                json,
            })),
        },
        Commands::Completions { .. } | Commands::ConfigFile { .. } => {
            Err(CliError::InvalidArgument(
                "This command should be handled before create_command is called".into(),
            ))
        }
    }
}
