use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::{self, Data, MeshPacket, PortNum};
use meshtastic::utils::generate_rand_id;
use serde::Serialize;

use super::{Command, CommandContext};

#[derive(Serialize)]
struct ReplyEventJson {
    event: String,
    from: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reply: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snr: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rssi: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hops: Option<u32>,
}

const BROADCAST_ADDR: u32 = 0xFFFFFFFF;

pub struct ReplyCommand;

#[async_trait]
impl Command for ReplyCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_node = ctx.node_db.my_node_num();
        let json = ctx.json;

        if !json {
            println!(
                "{} Auto-reply mode. Replying to text messages with signal info... Press {} to stop.\n",
                "->".cyan(),
                "Ctrl+C".bold()
            );
        }

        while let Some(packet) = ctx.packet_receiver.recv().await {
            let Some(PayloadVariant::Packet(ref mesh_packet)) = packet.payload_variant else {
                continue;
            };

            let Some(MeshPayload::Decoded(ref data)) = mesh_packet.payload_variant else {
                continue;
            };

            if data.portnum != PortNum::TextMessageApp as i32 {
                continue;
            }

            // Skip our own messages
            if mesh_packet.from == my_node {
                continue;
            }

            // Skip broadcast-only messages (no specific sender)
            if mesh_packet.from == BROADCAST_ADDR {
                continue;
            }

            let text = String::from_utf8(data.payload.clone())
                .unwrap_or_else(|_| "<invalid UTF-8>".to_string());

            let sender_name = ctx.node_db.node_name(mesh_packet.from);

            let snr = mesh_packet.rx_snr;
            let rssi = mesh_packet.rx_rssi;
            let hops = mesh_packet.hop_start.saturating_sub(mesh_packet.hop_limit);

            let reply_text = format!("ACK: SNR {:.1}dB, RSSI {}dBm, {} hop(s)", snr, rssi, hops);

            if !json {
                println!(
                    "{} From {} (!{:08x}): {}",
                    "<-".cyan(),
                    sender_name.unwrap_or("Unknown").to_string().bold(),
                    mesh_packet.from,
                    text
                );
            }

            let reply_packet = MeshPacket {
                from: my_node,
                to: mesh_packet.from,
                id: generate_rand_id(),
                want_ack: false,
                channel: mesh_packet.channel,
                hop_limit: 3,
                payload_variant: Some(MeshPayload::Decoded(Data {
                    portnum: PortNum::TextMessageApp as i32,
                    payload: reply_text.clone().into_bytes(),
                    ..Default::default()
                })),
                ..Default::default()
            };

            ctx.api
                .send_to_radio_packet(Some(protobufs::to_radio::PayloadVariant::Packet(
                    reply_packet,
                )))
                .await?;

            if json {
                let event = ReplyEventJson {
                    event: "reply".to_string(),
                    from: format!("!{:08x}", mesh_packet.from),
                    from_name: sender_name.map(|s| s.to_string()),
                    message: Some(text),
                    reply: Some(reply_text),
                    snr: Some(snr),
                    rssi: Some(rssi),
                    hops: Some(hops),
                };
                if let Ok(j) = serde_json::to_string(&event) {
                    println!("{}", j);
                }
            } else {
                println!(
                    "{} Reply to {} (!{:08x}): {}",
                    "->".green(),
                    sender_name.unwrap_or("Unknown"),
                    mesh_packet.from,
                    reply_text
                );
            }
        }

        println!("\nDisconnected from device.");
        Ok(())
    }
}
