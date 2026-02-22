use std::io::Write;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use colored::Colorize;
use crossterm::{cursor, execute, terminal};
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::NodeInfo;
use serde::Serialize;

use super::{Command, CommandContext};

pub struct WatchCommand {
    pub interval_secs: u64,
}

#[derive(Serialize)]
struct WatchNodeJson {
    id: String,
    name: String,
    battery: Option<u32>,
    snr: f32,
    last_heard: u32,
}

#[async_trait]
impl Command for WatchCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_node_num = ctx.node_db.my_node_num();
        let json = ctx.json;
        let mut interval = tokio::time::interval(Duration::from_secs(self.interval_secs));

        // Initial render
        if json {
            print_json_snapshot(ctx)?;
        } else {
            render_table_from_ctx(ctx, my_node_num)?;
        }

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if json {
                        print_json_snapshot(ctx)?;
                    } else {
                        render_table_from_ctx(ctx, my_node_num)?;
                    }
                }
                packet = ctx.packet_receiver.recv() => {
                    let Some(from_radio) = packet else {
                        break;
                    };
                    let Some(PayloadVariant::Packet(mesh_pkt)) = from_radio.payload_variant else {
                        continue;
                    };
                    let Some(MeshPayload::Decoded(ref data)) = mesh_pkt.payload_variant else {
                        continue;
                    };

                    let from = mesh_pkt.from;

                    // Update node_db with incoming data
                    update_node_from_packet(ctx, from, &mesh_pkt, data);
                }
            }
        }

        Ok(())
    }
}

fn update_node_from_packet(
    ctx: &mut CommandContext,
    from: u32,
    mesh_pkt: &meshtastic::protobufs::MeshPacket,
    data: &meshtastic::protobufs::Data,
) {
    // We can't directly mutate node_db entries, but we can update telemetry
    // by processing known packet types. The node_db doesn't expose mutable
    // access, so we track what we can.
    let _ = (ctx, from, mesh_pkt, data);
    // Node DB updates happen through the meshtastic crate's internal
    // packet processing. The watch command simply re-renders the current
    // node_db state periodically.
}

fn print_json_snapshot(ctx: &CommandContext) -> anyhow::Result<()> {
    let nodes = ctx.node_db.nodes();
    let mut sorted: Vec<_> = nodes.values().collect();
    sorted.sort_by_key(|n| n.num);

    let json_nodes: Vec<WatchNodeJson> = sorted
        .iter()
        .map(|n| WatchNodeJson {
            id: format!("!{:08x}", n.num),
            name: n
                .user
                .as_ref()
                .map(|u| u.long_name.clone())
                .unwrap_or_else(|| "Unknown".into()),
            battery: n.device_metrics.as_ref().and_then(|m| m.battery_level),
            snr: n.snr,
            last_heard: n.last_heard,
        })
        .collect();

    println!("{}", serde_json::to_string(&json_nodes)?);
    Ok(())
}

fn render_table_from_ctx(ctx: &CommandContext, my_node_num: u32) -> anyhow::Result<()> {
    let nodes = ctx.node_db.nodes();
    let mut stdout = std::io::stdout();

    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    let mut sorted: Vec<_> = nodes.values().collect();
    sorted.sort_by_key(|n| n.num);

    let header = format!(
        "{:<12} {:<20} {:<8} {:<8} {:<6} {:<20}",
        "ID", "Name", "Battery", "SNR", "Hops", "Last Heard"
    );
    writeln!(stdout, "{}", header.bold())?;
    writeln!(stdout, "{}", "-".repeat(76))?;

    for node in &sorted {
        let is_local = node.num == my_node_num;
        let line = format_node_row(node);

        if is_local {
            writeln!(stdout, "{}", line.green())?;
        } else {
            writeln!(stdout, "{}", line)?;
        }
    }

    writeln!(stdout)?;
    writeln!(
        stdout,
        "{}",
        format!("{} nodes | Press Ctrl+C to stop", sorted.len()).dimmed()
    )?;

    stdout.flush()?;
    Ok(())
}

fn format_node_row(node: &NodeInfo) -> String {
    let id = format!("!{:08x}", node.num);
    let name = node
        .user
        .as_ref()
        .map(|u| u.long_name.clone())
        .unwrap_or_else(|| "Unknown".into());
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

    format!(
        "{:<12} {:<20} {:<8} {:<8} {:<6} {:<20}",
        id,
        truncate(&name, 19),
        battery,
        snr,
        hops,
        last_heard
    )
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}…", &s[..max - 1])
    } else {
        s.to_string()
    }
}
