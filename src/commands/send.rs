use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::packet::PacketDestination;
use meshtastic::types::{MeshChannel, NodeId};

use super::{Command, CommandContext};
use crate::node_db::NodeDb;

pub enum DestinationSpec {
    Broadcast,
    NodeId(u32),
    NodeName(String),
}

pub struct SendCommand {
    pub message: String,
    pub destination: DestinationSpec,
    pub channel: MeshChannel,
}

#[async_trait]
impl Command for SendCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        ctx.api
            .send_text(
                &mut ctx.router,
                self.message.clone(),
                packet_dest,
                true,
                self.channel,
            )
            .await?;

        println!(
            "{} Message sent to {} on channel {}",
            "✓".green(),
            dest_label.bold(),
            self.channel.channel()
        );

        Ok(())
    }
}

fn resolve_destination(
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
