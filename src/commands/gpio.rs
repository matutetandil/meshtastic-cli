use std::time::{Duration, Instant};

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::{self, hardware_message, Data, HardwareMessage, MeshPacket, PortNum};
use meshtastic::utils::generate_rand_id;
use meshtastic::Message;
use serde::Serialize;

use super::{resolve_destination, Command, CommandContext, DestinationSpec};

#[derive(Serialize)]
struct GpioJson {
    event: String,
    mask: String,
    value: String,
    value_decimal: u64,
}

// ── GpioWriteCommand ─────────────────────────────────────────────

pub struct GpioWriteCommand {
    pub destination: DestinationSpec,
    pub mask: u64,
    pub value: u64,
}

#[async_trait]
impl Command for GpioWriteCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        let target_id = match packet_dest {
            meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
            _ => bail!("GPIO write requires a specific node destination"),
        };

        let msg = HardwareMessage {
            r#type: hardware_message::Type::WriteGpios as i32,
            gpio_mask: self.mask,
            gpio_value: self.value,
        };

        let my_node = ctx.node_db.my_node_num();

        let mesh_packet = MeshPacket {
            from: my_node,
            to: target_id,
            id: generate_rand_id(),
            want_ack: true,
            channel: 0,
            hop_limit: 3,
            payload_variant: Some(MeshPayload::Decoded(Data {
                portnum: PortNum::RemoteHardwareApp as i32,
                payload: msg.encode_to_vec(),
                ..Default::default()
            })),
            ..Default::default()
        };

        println!(
            "{} Writing GPIO on {}: mask=0x{:x}, value=0x{:x}",
            "->".cyan(),
            dest_label.bold(),
            self.mask,
            self.value
        );

        ctx.api
            .send_to_radio_packet(Some(protobufs::to_radio::PayloadVariant::Packet(
                mesh_packet,
            )))
            .await?;

        println!("{} GPIO write command sent.", "ok".green());

        Ok(())
    }
}

// ── GpioReadCommand ──────────────────────────────────────────────

pub struct GpioReadCommand {
    pub destination: DestinationSpec,
    pub mask: u64,
    pub timeout_secs: u64,
    pub json: bool,
}

#[async_trait]
impl Command for GpioReadCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        let target_id = match packet_dest {
            meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
            _ => bail!("GPIO read requires a specific node destination"),
        };

        let msg = HardwareMessage {
            r#type: hardware_message::Type::ReadGpios as i32,
            gpio_mask: self.mask,
            gpio_value: 0,
        };

        let my_node = ctx.node_db.my_node_num();
        let packet_id = generate_rand_id();

        let mesh_packet = MeshPacket {
            from: my_node,
            to: target_id,
            id: packet_id,
            want_ack: true,
            channel: 0,
            hop_limit: 3,
            payload_variant: Some(MeshPayload::Decoded(Data {
                portnum: PortNum::RemoteHardwareApp as i32,
                payload: msg.encode_to_vec(),
                want_response: true,
                ..Default::default()
            })),
            ..Default::default()
        };

        println!(
            "{} Reading GPIO from {}: mask=0x{:x}",
            "->".cyan(),
            dest_label.bold(),
            self.mask
        );

        ctx.api
            .send_to_radio_packet(Some(protobufs::to_radio::PayloadVariant::Packet(
                mesh_packet,
            )))
            .await?;

        let start = Instant::now();
        let timeout = Duration::from_secs(self.timeout_secs);

        loop {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                println!(
                    "{} Timeout after {}s — no GPIO response from {}",
                    "x".red(),
                    self.timeout_secs,
                    dest_label
                );
                return Ok(());
            }

            let packet = tokio::time::timeout(remaining, ctx.packet_receiver.recv()).await;

            match packet {
                Err(_) => {
                    println!(
                        "{} Timeout after {}s — no GPIO response from {}",
                        "x".red(),
                        self.timeout_secs,
                        dest_label
                    );
                    return Ok(());
                }
                Ok(None) => bail!("Packet receiver closed unexpectedly"),
                Ok(Some(envelope)) => {
                    let Some(PayloadVariant::Packet(mp)) = envelope.payload_variant else {
                        continue;
                    };
                    let Some(MeshPayload::Decoded(data)) = &mp.payload_variant else {
                        continue;
                    };
                    if data.portnum != PortNum::RemoteHardwareApp as i32 || mp.from != target_id {
                        continue;
                    }
                    if let Ok(hw_msg) = HardwareMessage::decode(data.payload.as_slice()) {
                        if hw_msg.r#type == hardware_message::Type::ReadGpiosReply as i32 {
                            if self.json {
                                let result = GpioJson {
                                    event: "read".to_string(),
                                    mask: format!("0x{:x}", hw_msg.gpio_mask),
                                    value: format!("0x{:x}", hw_msg.gpio_value),
                                    value_decimal: hw_msg.gpio_value,
                                };
                                println!("{}", serde_json::to_string_pretty(&result)?);
                            } else {
                                println!("{} GPIO read from {}:", "ok".green(), dest_label);
                                println!("  {:<16} 0x{:x}", "mask:".dimmed(), hw_msg.gpio_mask);
                                println!(
                                    "  {:<16} 0x{:x} ({})",
                                    "value:".dimmed(),
                                    hw_msg.gpio_value,
                                    hw_msg.gpio_value
                                );
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

// ── GpioWatchCommand ─────────────────────────────────────────────

pub struct GpioWatchCommand {
    pub destination: DestinationSpec,
    pub mask: u64,
    pub json: bool,
}

#[async_trait]
impl Command for GpioWatchCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        let target_id = match packet_dest {
            meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
            _ => bail!("GPIO watch requires a specific node destination"),
        };

        let msg = HardwareMessage {
            r#type: hardware_message::Type::WatchGpios as i32,
            gpio_mask: self.mask,
            gpio_value: 0,
        };

        let my_node = ctx.node_db.my_node_num();

        let mesh_packet = MeshPacket {
            from: my_node,
            to: target_id,
            id: generate_rand_id(),
            want_ack: true,
            channel: 0,
            hop_limit: 3,
            payload_variant: Some(MeshPayload::Decoded(Data {
                portnum: PortNum::RemoteHardwareApp as i32,
                payload: msg.encode_to_vec(),
                ..Default::default()
            })),
            ..Default::default()
        };

        println!(
            "{} Watching GPIO on {}: mask=0x{:x}... Press {} to stop.",
            "->".cyan(),
            dest_label.bold(),
            self.mask,
            "Ctrl+C".bold()
        );

        ctx.api
            .send_to_radio_packet(Some(protobufs::to_radio::PayloadVariant::Packet(
                mesh_packet,
            )))
            .await?;

        while let Some(envelope) = ctx.packet_receiver.recv().await {
            let Some(PayloadVariant::Packet(mp)) = envelope.payload_variant else {
                continue;
            };
            let Some(MeshPayload::Decoded(data)) = &mp.payload_variant else {
                continue;
            };
            if data.portnum != PortNum::RemoteHardwareApp as i32 || mp.from != target_id {
                continue;
            }
            if let Ok(hw_msg) = HardwareMessage::decode(data.payload.as_slice()) {
                if hw_msg.r#type == hardware_message::Type::GpiosChanged as i32 {
                    if self.json {
                        let event = GpioJson {
                            event: "changed".to_string(),
                            mask: format!("0x{:x}", hw_msg.gpio_mask),
                            value: format!("0x{:x}", hw_msg.gpio_value),
                            value_decimal: hw_msg.gpio_value,
                        };
                        if let Ok(j) = serde_json::to_string(&event) {
                            println!("{}", j);
                        }
                    } else {
                        println!(
                            "{} GPIO changed: mask=0x{:x}, value=0x{:x} ({})",
                            "!".yellow(),
                            hw_msg.gpio_mask,
                            hw_msg.gpio_value,
                            hw_msg.gpio_value
                        );
                    }
                }
            }
        }

        println!("\nDisconnected from device.");
        Ok(())
    }
}

// ── Helpers ──────────────────────────────────────────────────────

pub fn parse_bitmask(s: &str) -> anyhow::Result<u64> {
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u64::from_str_radix(hex, 16).map_err(|_| anyhow::anyhow!("Invalid hex bitmask: '{}'", s))
    } else {
        s.parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid bitmask '{}'. Use decimal or 0x hex.", s))
    }
}
