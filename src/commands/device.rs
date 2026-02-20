use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::{self, admin_message, mesh_packet, Data, MeshPacket, PortNum};
use meshtastic::utils::generate_rand_id;
use meshtastic::Message;

use super::{resolve_destination, Command, CommandContext, DestinationSpec};

// ── RebootCommand ─────────────────────────────────────────────────

pub struct RebootCommand {
    pub destination: DestinationSpec,
    pub delay_secs: i32,
}

#[async_trait]
impl Command for RebootCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let (target_id, label) = resolve_target(&self.destination, &ctx)?;

        println!(
            "{} Rebooting {} in {} seconds...",
            "->".cyan(),
            label.bold(),
            self.delay_secs
        );

        send_admin_message(
            &mut ctx,
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
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let (target_id, label) = resolve_target(&self.destination, &ctx)?;

        println!(
            "{} Shutting down {} in {} seconds...",
            "->".cyan(),
            label.bold(),
            self.delay_secs
        );

        send_admin_message(
            &mut ctx,
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
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();
        let label = format!("local device (!{:08x})", my_id);

        println!("{} Factory resetting {}...", "->".cyan(), label.bold());
        println!(
            "  {} All settings and state will be restored to defaults.",
            "!".yellow().bold()
        );

        send_admin_message(
            &mut ctx,
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
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();
        let label = format!("local device (!{:08x})", my_id);

        println!("{} Resetting NodeDB on {}...", "->".cyan(), label.bold());

        send_admin_message(
            &mut ctx,
            my_id,
            admin_message::PayloadVariant::NodedbReset(5),
        )
        .await?;

        println!("{} NodeDB reset command sent.", "ok".green());

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

pub(super) async fn send_admin_message(
    ctx: &mut CommandContext,
    target_id: u32,
    payload: admin_message::PayloadVariant,
) -> anyhow::Result<()> {
    let admin_msg = protobufs::AdminMessage {
        payload_variant: Some(payload),
        session_passkey: Vec::new(),
    };

    let mesh_packet = MeshPacket {
        from: ctx.node_db.my_node_num(),
        to: target_id,
        id: generate_rand_id(),
        want_ack: true,
        channel: 0,
        hop_limit: 3,
        payload_variant: Some(mesh_packet::PayloadVariant::Decoded(Data {
            portnum: PortNum::AdminApp as i32,
            payload: admin_msg.encode_to_vec(),
            want_response: false,
            ..Default::default()
        })),
        ..Default::default()
    };

    let payload_variant = Some(protobufs::to_radio::PayloadVariant::Packet(mesh_packet));
    ctx.api.send_to_radio_packet(payload_variant).await?;

    Ok(())
}
