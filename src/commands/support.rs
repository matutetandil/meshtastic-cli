use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::{self, HardwareModel};

use super::{Command, CommandContext};

pub struct SupportCommand;

#[async_trait]
impl Command for SupportCommand {
    async fn execute(self: Box<Self>, ctx: CommandContext) -> anyhow::Result<()> {
        println!("{}", "Support Info".bold().underline());
        println!();

        // CLI version
        println!(
            "  {:<28} {}",
            "cli_version:".dimmed(),
            env!("CARGO_PKG_VERSION")
        );

        // Firmware and hardware from metadata
        if let Some(meta) = ctx.node_db.metadata() {
            println!(
                "  {:<28} {}",
                "firmware_version:".dimmed(),
                meta.firmware_version
            );

            let hw_model = HardwareModel::try_from(meta.hw_model)
                .map(|m| m.as_str_name().to_string())
                .unwrap_or_else(|_| format!("Unknown({})", meta.hw_model));
            println!("  {:<28} {}", "hw_model:".dimmed(), hw_model);

            let role = protobufs::config::device_config::Role::try_from(meta.role)
                .map(|r| format!("{:?}", r))
                .unwrap_or_else(|_| meta.role.to_string());
            println!("  {:<28} {}", "role:".dimmed(), role);

            println!("  {:<28} {}", "has_wifi:".dimmed(), meta.has_wifi);
            println!("  {:<28} {}", "has_bluetooth:".dimmed(), meta.has_bluetooth);
            println!("  {:<28} {}", "has_ethernet:".dimmed(), meta.has_ethernet);
        } else {
            println!("  {}", "(no device metadata available)".dimmed());
        }

        // Node ID
        let my_id = ctx.node_db.my_node_num();
        println!("  {:<28} !{:08x}", "node_id:".dimmed(), my_id);

        // Region and modem preset from LoRa config
        let lora = ctx.node_db.local_config().lora.clone();
        if let Some(lora) = lora {
            let region = protobufs::config::lo_ra_config::RegionCode::try_from(lora.region)
                .map(|r| format!("{:?}", r))
                .unwrap_or_else(|_| lora.region.to_string());
            println!("  {:<28} {}", "region:".dimmed(), region);

            let preset = protobufs::config::lo_ra_config::ModemPreset::try_from(lora.modem_preset)
                .map(|p| format!("{:?}", p))
                .unwrap_or_else(|_| lora.modem_preset.to_string());
            println!("  {:<28} {}", "modem_preset:".dimmed(), preset);
        }

        // Channel and node counts
        let active_channels = ctx
            .node_db
            .channels()
            .iter()
            .filter(|ch| ch.role != protobufs::channel::Role::Disabled as i32)
            .count();
        println!("  {:<28} {}", "active_channels:".dimmed(), active_channels);
        println!(
            "  {:<28} {}",
            "known_nodes:".dimmed(),
            ctx.node_db.nodes().len()
        );

        println!();

        Ok(())
    }
}
