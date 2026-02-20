use async_trait::async_trait;
use chrono::{DateTime, Utc};
use colored::Colorize;

use super::{Command, CommandContext};

pub struct NodesCommand;

#[async_trait]
impl Command for NodesCommand {
    async fn execute(self: Box<Self>, ctx: CommandContext) -> anyhow::Result<()> {
        let local_node_num = ctx.node_db.my_node_num();
        let nodes = ctx.node_db.nodes();

        if nodes.is_empty() {
            println!("No nodes found in mesh.");
            return Ok(());
        }

        let header = format!(
            "{:<12} {:<20} {:<8} {:<8} {:<6} {}",
            "ID", "Name", "Battery", "SNR", "Hops", "Last Heard"
        );
        println!("{header}");
        let separator = "-".repeat(72);
        println!("{separator}");

        let mut sorted_nodes: Vec<_> = nodes.values().collect();
        sorted_nodes.sort_by_key(|n| n.num);

        for node in sorted_nodes {
            let is_local = node.num == local_node_num;

            let id = format!("!{:08x}", node.num);

            let name = node
                .user
                .as_ref()
                .map(|u| u.long_name.as_str())
                .unwrap_or("Unknown");

            let battery = node
                .device_metrics
                .as_ref()
                .and_then(|m| m.battery_level)
                .map(|b| format!("{}%", b))
                .unwrap_or_else(|| "N/A".into());

            let snr = if node.snr != 0.0 {
                format!("{:.1}", node.snr)
            } else {
                "N/A".into()
            };

            let hops = node
                .hops_away
                .map(|h| h.to_string())
                .unwrap_or_else(|| "N/A".into());

            let last_heard = if node.last_heard > 0 {
                DateTime::<Utc>::from_timestamp(node.last_heard as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Invalid".into())
            } else {
                "Never".into()
            };

            let line = format!(
                "{:<12} {:<20} {:<8} {:<8} {:<6} {}",
                id, name, battery, snr, hops, last_heard
            );

            if is_local {
                println!("{}", line.green());
            } else {
                println!("{}", line);
            }
        }

        Ok(())
    }
}
