use std::time::{Duration, Instant};

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::{self, routing, Data, MeshPacket, PortNum, Routing};
use meshtastic::types::MeshChannel;
use meshtastic::utils::generate_rand_id;
use meshtastic::Message;

use super::{resolve_destination, Command, CommandContext, DestinationSpec};

pub struct SendCommand {
    pub message: String,
    pub destination: DestinationSpec,
    pub channel: MeshChannel,
    pub wait_ack: bool,
    pub timeout_secs: u64,
    pub private: bool,
}

#[async_trait]
impl Command for SendCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        if self.wait_ack && matches!(self.destination, DestinationSpec::Broadcast) {
            bail!("--ack requires a specific destination (use --dest or --to)");
        }

        let needs_manual_packet = self.wait_ack || self.private;

        if needs_manual_packet {
            let target_id = match packet_dest {
                meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
                _ => 0xFFFFFFFF,
            };

            let packet_id = generate_rand_id();
            let my_node = ctx.node_db.my_node_num();

            let portnum = if self.private {
                PortNum::PrivateApp
            } else {
                PortNum::TextMessageApp
            };

            let mesh_packet = MeshPacket {
                from: my_node,
                to: target_id,
                id: packet_id,
                want_ack: true,
                channel: self.channel.channel(),
                hop_limit: 3,
                payload_variant: Some(MeshPayload::Decoded(Data {
                    portnum: portnum as i32,
                    payload: self.message.clone().into_bytes(),
                    ..Default::default()
                })),
                ..Default::default()
            };

            ctx.api
                .send_to_radio_packet(Some(protobufs::to_radio::PayloadVariant::Packet(
                    mesh_packet,
                )))
                .await?;

            let port_label = if self.private { " (private)" } else { "" };
            println!(
                "{} Message{} sent to {} on channel {}",
                "ok".green(),
                port_label,
                dest_label.bold(),
                self.channel.channel()
            );

            if self.wait_ack {
                println!("{} Waiting for ACK...", "->".cyan());
                wait_for_ack(&mut ctx, packet_id, self.timeout_secs, &dest_label).await?;
            }
        } else {
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
                "ok".green(),
                dest_label.bold(),
                self.channel.channel()
            );
        }

        Ok(())
    }
}

async fn wait_for_ack(
    ctx: &mut CommandContext,
    packet_id: u32,
    timeout_secs: u64,
    dest_label: &str,
) -> anyhow::Result<()> {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    loop {
        let remaining = timeout.saturating_sub(start.elapsed());
        if remaining.is_zero() {
            println!(
                "{} Timeout after {}s — no ACK from {}",
                "x".red(),
                timeout_secs,
                dest_label
            );
            return Ok(());
        }

        let packet = tokio::time::timeout(remaining, ctx.packet_receiver.recv()).await;

        match packet {
            Err(_) => {
                println!(
                    "{} Timeout after {}s — no ACK from {}",
                    "x".red(),
                    timeout_secs,
                    dest_label
                );
                return Ok(());
            }
            Ok(None) => bail!("Disconnected while waiting for ACK"),
            Ok(Some(from_radio)) => {
                let Some(PayloadVariant::Packet(mesh_pkt)) = from_radio.payload_variant else {
                    continue;
                };
                let Some(MeshPayload::Decoded(ref data)) = mesh_pkt.payload_variant else {
                    continue;
                };
                if data.portnum != PortNum::RoutingApp as i32 || data.request_id != packet_id {
                    continue;
                }

                let rtt = start.elapsed();
                let Ok(routing_msg) = Routing::decode(data.payload.as_slice()) else {
                    println!(
                        "{} Received routing response but failed to decode",
                        "?".yellow()
                    );
                    return Ok(());
                };

                match routing_msg.variant {
                    Some(routing::Variant::ErrorReason(0)) => {
                        println!(
                            "{} ACK from {} in {:.1}s",
                            "ok".green(),
                            dest_label.bold(),
                            rtt.as_secs_f64()
                        );
                    }
                    Some(routing::Variant::ErrorReason(code)) => {
                        let reason = routing::Error::try_from(code)
                            .map(|e| format!("{:?}", e))
                            .unwrap_or_else(|_| format!("code {}", code));
                        println!(
                            "{} NAK from {}: {} ({:.1}s)",
                            "x".red(),
                            dest_label,
                            reason,
                            rtt.as_secs_f64()
                        );
                    }
                    _ => {
                        println!(
                            "{} Unexpected routing response ({:.1}s)",
                            "?".yellow(),
                            rtt.as_secs_f64()
                        );
                    }
                }

                return Ok(());
            }
        }
    }
}
