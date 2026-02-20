mod listen;
mod nodes;
mod send;

use async_trait::async_trait;
use meshtastic::api::ConnectedStreamApi;
use meshtastic::packet::PacketReceiver;
use meshtastic::types::MeshChannel;

use crate::cli::Commands;
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

pub fn create_command(command: &Commands) -> Result<Box<dyn Command>, CliError> {
    match command {
        Commands::Nodes => Ok(Box::new(nodes::NodesCommand)),
        Commands::Listen => Ok(Box::new(listen::ListenCommand)),
        Commands::Send {
            message,
            dest,
            to,
            channel,
        } => {
            let destination = match (dest, to) {
                (Some(hex_str), None) => {
                    let stripped = hex_str.strip_prefix('!').unwrap_or(hex_str);
                    let node_num = u32::from_str_radix(stripped, 16).map_err(|_| {
                        CliError::InvalidArgument(format!(
                            "Invalid node ID '{}'. Expected hex format like !abcd1234",
                            hex_str
                        ))
                    })?;
                    send::DestinationSpec::NodeId(node_num)
                }
                (None, Some(name)) => send::DestinationSpec::NodeName(name.clone()),
                _ => send::DestinationSpec::Broadcast,
            };

            let mesh_channel = MeshChannel::new(*channel)
                .map_err(|e| CliError::InvalidArgument(format!("Invalid channel index: {}", e)))?;

            Ok(Box::new(send::SendCommand {
                message: message.clone(),
                destination,
                channel: mesh_channel,
            }))
        }
    }
}
