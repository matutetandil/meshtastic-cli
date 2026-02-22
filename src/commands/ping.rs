use std::time::{Duration, Instant};

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::{self, routing, Data, MeshPacket, PortNum, Routing};
use meshtastic::utils::generate_rand_id;
use meshtastic::Message;
use serde::Serialize;

use super::{resolve_destination, Command, CommandContext, DestinationSpec};

#[derive(Serialize)]
struct PingJson {
    dest: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    rtt_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

pub struct PingCommand {
    pub destination: DestinationSpec,
    pub timeout_secs: u64,
    pub json: bool,
}

#[async_trait]
impl Command for PingCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        let target_node_id = match packet_dest {
            meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
            _ => bail!("Ping requires a specific node destination"),
        };

        let packet_id: u32 = generate_rand_id();
        let my_node_num = ctx.node_db.my_node_num();

        let mesh_packet = MeshPacket {
            from: my_node_num,
            to: target_node_id,
            id: packet_id,
            want_ack: true,
            channel: 0,
            hop_limit: 3,
            payload_variant: Some(MeshPayload::Decoded(Data {
                portnum: PortNum::TextMessageApp as i32,
                payload: "ping".to_string().into_bytes(),
                ..Default::default()
            })),
            ..Default::default()
        };

        let payload_variant = Some(protobufs::to_radio::PayloadVariant::Packet(mesh_packet));

        println!(
            "{} Pinging {} (packet id: {:08x})...",
            "→".cyan(),
            dest_label.bold(),
            packet_id
        );

        let start = Instant::now();

        ctx.api.send_to_radio_packet(payload_variant).await?;

        let timeout = Duration::from_secs(self.timeout_secs);

        loop {
            let packet =
                tokio::time::timeout(timeout - start.elapsed(), ctx.packet_receiver.recv()).await;

            match packet {
                Err(_) => {
                    if self.json {
                        let result = PingJson {
                            dest: dest_label.to_string(),
                            status: "timeout".to_string(),
                            rtt_ms: None,
                            error: Some(format!("Timeout after {}s", self.timeout_secs)),
                        };
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!(
                            "{} Timeout after {}s — no ACK from {}",
                            "✗".red(),
                            self.timeout_secs,
                            dest_label
                        );
                    }
                    return Ok(());
                }
                Ok(None) => {
                    bail!("Disconnected while waiting for ACK");
                }
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

                    if self.json {
                        let result = match routing_msg.variant {
                            Some(routing::Variant::ErrorReason(0)) => PingJson {
                                dest: dest_label.to_string(),
                                status: "ack".to_string(),
                                rtt_ms: Some(rtt.as_millis()),
                                error: None,
                            },
                            Some(routing::Variant::ErrorReason(code)) => {
                                let reason = routing::Error::try_from(code)
                                    .map(|e| format!("{:?}", e))
                                    .unwrap_or_else(|_| format!("code {}", code));
                                PingJson {
                                    dest: dest_label.to_string(),
                                    status: "nak".to_string(),
                                    rtt_ms: Some(rtt.as_millis()),
                                    error: Some(reason),
                                }
                            }
                            _ => PingJson {
                                dest: dest_label.to_string(),
                                status: "unknown".to_string(),
                                rtt_ms: Some(rtt.as_millis()),
                                error: None,
                            },
                        };
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        match routing_msg.variant {
                            Some(routing::Variant::ErrorReason(0)) => {
                                println!(
                                    "{} ACK from {} in {:.1}s",
                                    "✓".green(),
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
                                    "✗".red(),
                                    dest_label,
                                    reason,
                                    rtt.as_secs_f64()
                                );
                            }
                            _ => {
                                println!(
                                    "{} Unexpected routing response from {} ({:.1}s)",
                                    "?".yellow(),
                                    dest_label,
                                    rtt.as_secs_f64()
                                );
                            }
                        }
                    }

                    return Ok(());
                }
            }
        }
    }
}
