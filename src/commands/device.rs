use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::{
    self, admin_message, mesh_packet, AdminMessage, Data, MeshPacket, PortNum,
};
use meshtastic::utils::generate_rand_id;
use meshtastic::Message;

use super::admin::send_admin_message;
use super::{resolve_destination, Command, CommandContext, DestinationSpec};

// ── RebootCommand ─────────────────────────────────────────────────

pub struct RebootCommand {
    pub destination: DestinationSpec,
    pub delay_secs: i32,
}

#[async_trait]
impl Command for RebootCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (target_id, label) = resolve_target(&self.destination, ctx)?;

        println!(
            "{} Rebooting {} in {} seconds...",
            "->".cyan(),
            label.bold(),
            self.delay_secs
        );

        send_admin_message(
            ctx,
            target_id,
            admin_message::PayloadVariant::RebootSeconds(self.delay_secs),
        )
        .await?;

        println!("{} Reboot command sent to {}.", "ok".green(), label);

        Ok(())
    }
}

// ── ShutdownCommand ───────────────────────────────────────────────

pub struct ShutdownCommand {
    pub destination: DestinationSpec,
    pub delay_secs: i32,
}

#[async_trait]
impl Command for ShutdownCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (target_id, label) = resolve_target(&self.destination, ctx)?;

        println!(
            "{} Shutting down {} in {} seconds...",
            "->".cyan(),
            label.bold(),
            self.delay_secs
        );

        send_admin_message(
            ctx,
            target_id,
            admin_message::PayloadVariant::ShutdownSeconds(self.delay_secs),
        )
        .await?;

        println!("{} Shutdown command sent to {}.", "ok".green(), label);

        Ok(())
    }
}

// ── FactoryResetCommand ───────────────────────────────────────────

pub struct FactoryResetCommand;

#[async_trait]
impl Command for FactoryResetCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();
        let label = format!("local device (!{:08x})", my_id);

        println!("{} Factory resetting {}...", "->".cyan(), label.bold());
        println!(
            "  {} All settings and state will be restored to defaults.",
            "!".yellow().bold()
        );

        send_admin_message(
            ctx,
            my_id,
            admin_message::PayloadVariant::FactoryResetConfig(5),
        )
        .await?;

        println!("{} Factory reset command sent.", "ok".green());

        Ok(())
    }
}

// ── ResetNodeDbCommand ────────────────────────────────────────────

pub struct ResetNodeDbCommand;

#[async_trait]
impl Command for ResetNodeDbCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();
        let label = format!("local device (!{:08x})", my_id);

        println!("{} Resetting NodeDB on {}...", "->".cyan(), label.bold());

        send_admin_message(ctx, my_id, admin_message::PayloadVariant::NodedbReset(5)).await?;

        println!("{} NodeDB reset command sent.", "ok".green());

        Ok(())
    }
}

// ── SetTimeCommand ────────────────────────────────────────────────

pub struct SetTimeCommand {
    pub time: Option<u32>,
}

#[async_trait]
impl Command for SetTimeCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let timestamp = self.time.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as u32
        });

        let my_id = ctx.node_db.my_node_num();

        println!(
            "{} Setting device time to {} (unix timestamp)...",
            "->".cyan(),
            timestamp
        );

        send_admin_message(
            ctx,
            my_id,
            admin_message::PayloadVariant::SetTimeOnly(timestamp),
        )
        .await?;

        println!("{} Device time set.", "ok".green());
        Ok(())
    }
}

// ── SetCannedMessageCommand ──────────────────────────────────────

pub struct SetCannedMessageCommand {
    pub message: String,
}

#[async_trait]
impl Command for SetCannedMessageCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();
        let count = self.message.split('|').count();

        println!("{} Setting {} canned message(s)...", "->".cyan(), count);

        send_admin_message(
            ctx,
            my_id,
            admin_message::PayloadVariant::SetCannedMessageModuleMessages(self.message.clone()),
        )
        .await?;

        println!("{} Canned messages updated.", "ok".green());
        Ok(())
    }
}

// ── GetCannedMessageCommand ──────────────────────────────────────

pub struct GetCannedMessageCommand {
    pub timeout_secs: u64,
}

#[async_trait]
impl Command for GetCannedMessageCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();
        let packet_id = generate_rand_id();

        let admin_msg = AdminMessage {
            payload_variant: Some(
                admin_message::PayloadVariant::GetCannedMessageModuleMessagesRequest(true),
            ),
            session_passkey: Vec::new(),
        };

        let mesh_packet = MeshPacket {
            from: my_id,
            to: my_id,
            id: packet_id,
            want_ack: true,
            channel: 0,
            hop_limit: 3,
            payload_variant: Some(mesh_packet::PayloadVariant::Decoded(Data {
                portnum: PortNum::AdminApp as i32,
                payload: admin_msg.encode_to_vec(),
                want_response: true,
                ..Default::default()
            })),
            ..Default::default()
        };

        println!("{} Requesting canned messages...", "->".cyan());

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
                    "{} Timeout after {}s — no response.",
                    "x".red(),
                    self.timeout_secs
                );
                return Ok(());
            }

            let packet = tokio::time::timeout(remaining, ctx.packet_receiver.recv()).await;

            match packet {
                Err(_) => {
                    println!(
                        "{} Timeout after {}s — no response.",
                        "x".red(),
                        self.timeout_secs
                    );
                    return Ok(());
                }
                Ok(None) => anyhow::bail!("Packet receiver closed unexpectedly"),
                Ok(Some(envelope)) => {
                    let Some(PayloadVariant::Packet(mp)) = envelope.payload_variant else {
                        continue;
                    };
                    let Some(MeshPayload::Decoded(data)) = &mp.payload_variant else {
                        continue;
                    };
                    if data.portnum != PortNum::AdminApp as i32 || mp.from != my_id {
                        continue;
                    }
                    if let Ok(admin) = AdminMessage::decode(data.payload.as_slice()) {
                        if let Some(
                            admin_message::PayloadVariant::GetCannedMessageModuleMessagesResponse(
                                messages,
                            ),
                        ) = admin.payload_variant
                        {
                            println!("{}", "Canned Messages".bold().underline());
                            if messages.is_empty() {
                                println!("  {}", "(none configured)".dimmed());
                            } else {
                                for (i, msg) in messages.split('|').enumerate() {
                                    println!("  {}: {}", format!("[{}]", i).bold(), msg);
                                }
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

// ── SetRingtoneCommand ───────────────────────────────────────────

pub struct SetRingtoneCommand {
    pub ringtone: String,
}

#[async_trait]
impl Command for SetRingtoneCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();

        println!("{} Setting ringtone: {}", "->".cyan(), self.ringtone.bold());

        send_admin_message(
            ctx,
            my_id,
            admin_message::PayloadVariant::SetRingtoneMessage(self.ringtone.clone()),
        )
        .await?;

        println!("{} Ringtone updated.", "ok".green());
        Ok(())
    }
}

// ── GetRingtoneCommand ──────────────────────────────────────────

pub struct GetRingtoneCommand {
    pub timeout_secs: u64,
}

#[async_trait]
impl Command for GetRingtoneCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();
        let packet_id = generate_rand_id();

        let admin_msg = AdminMessage {
            payload_variant: Some(admin_message::PayloadVariant::GetRingtoneRequest(true)),
            session_passkey: Vec::new(),
        };

        let mesh_packet = MeshPacket {
            from: my_id,
            to: my_id,
            id: packet_id,
            want_ack: true,
            channel: 0,
            hop_limit: 3,
            payload_variant: Some(mesh_packet::PayloadVariant::Decoded(Data {
                portnum: PortNum::AdminApp as i32,
                payload: admin_msg.encode_to_vec(),
                want_response: true,
                ..Default::default()
            })),
            ..Default::default()
        };

        println!("{} Requesting ringtone...", "->".cyan());

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
                    "{} Timeout after {}s — no response.",
                    "x".red(),
                    self.timeout_secs
                );
                return Ok(());
            }

            let packet = tokio::time::timeout(remaining, ctx.packet_receiver.recv()).await;

            match packet {
                Err(_) => {
                    println!(
                        "{} Timeout after {}s — no response.",
                        "x".red(),
                        self.timeout_secs
                    );
                    return Ok(());
                }
                Ok(None) => anyhow::bail!("Packet receiver closed unexpectedly"),
                Ok(Some(envelope)) => {
                    let Some(PayloadVariant::Packet(mp)) = envelope.payload_variant else {
                        continue;
                    };
                    let Some(MeshPayload::Decoded(data)) = &mp.payload_variant else {
                        continue;
                    };
                    if data.portnum != PortNum::AdminApp as i32 || mp.from != my_id {
                        continue;
                    }
                    if let Ok(admin) = AdminMessage::decode(data.payload.as_slice()) {
                        if let Some(admin_message::PayloadVariant::GetRingtoneResponse(ringtone)) =
                            admin.payload_variant
                        {
                            println!("{}", "Ringtone".bold().underline());
                            if ringtone.is_empty() {
                                println!("  {}", "(none configured)".dimmed());
                            } else {
                                println!("  {}", ringtone);
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

// ── RebootOtaCommand ─────────────────────────────────────────────

pub struct RebootOtaCommand {
    pub destination: DestinationSpec,
    pub delay_secs: i32,
}

#[async_trait]
impl Command for RebootOtaCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (target_id, label) = resolve_target(&self.destination, ctx)?;

        println!(
            "{} Rebooting {} into OTA mode in {} seconds...",
            "->".cyan(),
            label.bold(),
            self.delay_secs
        );

        send_admin_message(
            ctx,
            target_id,
            admin_message::PayloadVariant::RebootOtaSeconds(self.delay_secs),
        )
        .await?;

        println!("{} OTA reboot command sent to {}.", "ok".green(), label);

        Ok(())
    }
}

// ── EnterDfuCommand ──────────────────────────────────────────────

pub struct EnterDfuCommand;

#[async_trait]
impl Command for EnterDfuCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();
        let label = format!("local device (!{:08x})", my_id);

        println!("{} Entering DFU mode on {}...", "->".cyan(), label.bold());

        send_admin_message(
            ctx,
            my_id,
            admin_message::PayloadVariant::EnterDfuModeRequest(true),
        )
        .await?;

        println!("{} DFU mode command sent.", "ok".green());

        Ok(())
    }
}

// ── FactoryResetDeviceCommand ────────────────────────────────────

pub struct FactoryResetDeviceCommand;

#[async_trait]
impl Command for FactoryResetDeviceCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();
        let label = format!("local device (!{:08x})", my_id);

        println!("{} Full factory reset on {}...", "->".cyan(), label.bold());
        println!(
            "  {} This will erase ALL settings including BLE bonds.",
            "!".yellow().bold()
        );

        send_admin_message(
            ctx,
            my_id,
            admin_message::PayloadVariant::FactoryResetDevice(5),
        )
        .await?;

        println!("{} Full factory reset command sent.", "ok".green());

        Ok(())
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn resolve_target(
    destination: &DestinationSpec,
    ctx: &CommandContext,
) -> anyhow::Result<(u32, String)> {
    match destination {
        DestinationSpec::Broadcast => {
            let my_id = ctx.node_db.my_node_num();
            let label = format!("local device (!{:08x})", my_id);
            Ok((my_id, label))
        }
        _ => {
            let (packet_dest, label) = resolve_destination(destination, &ctx.node_db)?;
            let target_id = match packet_dest {
                meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
                _ => ctx.node_db.my_node_num(),
            };
            Ok((target_id, label))
        }
    }
}
