mod nodes;

use async_trait::async_trait;
use meshtastic::api::ConnectedStreamApi;
use meshtastic::packet::PacketReceiver;

use crate::cli::Commands;
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

pub fn create_command(command: &Commands) -> Box<dyn Command> {
    match command {
        Commands::Nodes => Box::new(nodes::NodesCommand),
    }
}
