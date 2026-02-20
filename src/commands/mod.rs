mod channel;
mod config;
mod export_import;
mod info;
mod listen;
mod nodes;
mod ping;
mod send;
mod traceroute;

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::api::ConnectedStreamApi;
use meshtastic::packet::{PacketDestination, PacketReceiver};
use meshtastic::types::{MeshChannel, NodeId};

use crate::cli::{ChannelAction, Commands, ConfigAction};
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
        Commands::Nodes => Ok(Box::new(nodes::NodesCommand)),
        Commands::Listen => Ok(Box::new(listen::ListenCommand)),
        Commands::Info => Ok(Box::new(info::InfoCommand)),
        Commands::Send {
            message,
            dest,
            to,
            channel,
        } => {
            let destination = parse_dest_spec(dest, to)?;
            let mesh_channel = MeshChannel::new(*channel)
                .map_err(|e| CliError::InvalidArgument(format!("Invalid channel index: {}", e)))?;

            Ok(Box::new(send::SendCommand {
                message: message.clone(),
                destination,
                channel: mesh_channel,
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
        },
    }
}
