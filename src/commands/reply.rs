use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::{self, Data, MeshPacket, PortNum};
use meshtastic::utils::generate_rand_id;

use super::{Command, CommandContext};

const BROADCAST_ADDR: u32 = 0xFFFFFFFF;

pub struct ReplyCommand;

#[async_trait]
impl Command for ReplyCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let my_node = ctx.node_db.my_node_num();

        println!(
            "{} Auto-reply mode. Replying to text messages with signal info... Press {} to stop.\n",
            "->".cyan(),
            "Ctrl+C".bold()
        );

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

            let sender_name = ctx.node_db.node_name(mesh_packet.from).unwrap_or("Unknown");

            println!(
                "{} From {} (!{:08x}): {}",
                "<-".cyan(),
                sender_name.bold(),
                mesh_packet.from,
                text
            );

            let snr = mesh_packet.rx_snr;
            let rssi = mesh_packet.rx_rssi;
            let hops = mesh_packet.hop_start.saturating_sub(mesh_packet.hop_limit);

            let reply_text = format!("ACK: SNR {:.1}dB, RSSI {}dBm, {} hop(s)", snr, rssi, hops);

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

            println!(
                "{} Reply to {} (!{:08x}): {}",
                "->".green(),
                sender_name,
                mesh_packet.from,
                reply_text
            );
        }

        println!("\nDisconnected from device.");
        Ok(())
    }
}
