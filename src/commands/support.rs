use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::{self, HardwareModel};
use serde::Serialize;

use super::{Command, CommandContext};

pub struct SupportCommand {
    pub json: bool,
}

#[derive(Serialize)]
struct SupportJson {
    cli_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    firmware_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hw_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_wifi: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_bluetooth: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_ethernet: Option<bool>,
    node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    modem_preset: Option<String>,
    active_channels: usize,
    known_nodes: usize,
}

#[async_trait]
impl Command for SupportCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let meta = ctx.node_db.metadata();
        let my_id = ctx.node_db.my_node_num();
        let lora = ctx.node_db.local_config().lora.clone();

        let active_channels = ctx
            .node_db
            .channels()
            .iter()
            .filter(|ch| ch.role != protobufs::channel::Role::Disabled as i32)
            .count();
        let known_nodes = ctx.node_db.nodes().len();

        let hw_model_str = meta.map(|m| {
            HardwareModel::try_from(m.hw_model)
                .map(|h| h.as_str_name().to_string())
                .unwrap_or_else(|_| format!("Unknown({})", m.hw_model))
        });

        let role_str = meta.map(|m| {
            protobufs::config::device_config::Role::try_from(m.role)
                .map(|r| format!("{:?}", r))
                .unwrap_or_else(|_| m.role.to_string())
        });

        let region_str = lora.as_ref().map(|l| {
            protobufs::config::lo_ra_config::RegionCode::try_from(l.region)
                .map(|r| format!("{:?}", r))
                .unwrap_or_else(|_| l.region.to_string())
        });

        let preset_str = lora.as_ref().map(|l| {
            protobufs::config::lo_ra_config::ModemPreset::try_from(l.modem_preset)
                .map(|p| format!("{:?}", p))
                .unwrap_or_else(|_| l.modem_preset.to_string())
        });

        if self.json {
            let info = SupportJson {
                cli_version: env!("CARGO_PKG_VERSION").to_string(),
                firmware_version: meta.map(|m| m.firmware_version.clone()),
                hw_model: hw_model_str,
                role: role_str,
                has_wifi: meta.map(|m| m.has_wifi),
                has_bluetooth: meta.map(|m| m.has_bluetooth),
                has_ethernet: meta.map(|m| m.has_ethernet),
                node_id: format!("!{:08x}", my_id),
                region: region_str,
                modem_preset: preset_str,
                active_channels,
                known_nodes,
            };
            println!("{}", serde_json::to_string_pretty(&info)?);
            return Ok(());
        }

        println!("{}", "Support Info".bold().underline());
        println!();

        println!(
            "  {:<28} {}",
            "cli_version:".dimmed(),
            env!("CARGO_PKG_VERSION")
        );

        if let Some(meta) = meta {
            println!(
                "  {:<28} {}",
                "firmware_version:".dimmed(),
                meta.firmware_version
            );
            println!(
                "  {:<28} {}",
                "hw_model:".dimmed(),
                hw_model_str.as_deref().unwrap_or("Unknown")
            );
            println!(
                "  {:<28} {}",
                "role:".dimmed(),
                role_str.as_deref().unwrap_or("Unknown")
            );
            println!("  {:<28} {}", "has_wifi:".dimmed(), meta.has_wifi);
            println!("  {:<28} {}", "has_bluetooth:".dimmed(), meta.has_bluetooth);
            println!("  {:<28} {}", "has_ethernet:".dimmed(), meta.has_ethernet);
        } else {
            println!("  {}", "(no device metadata available)".dimmed());
        }

        println!("  {:<28} !{:08x}", "node_id:".dimmed(), my_id);

        if let Some(region) = &region_str {
            println!("  {:<28} {}", "region:".dimmed(), region);
        }
        if let Some(preset) = &preset_str {
            println!("  {:<28} {}", "modem_preset:".dimmed(), preset);
        }

        println!("  {:<28} {}", "active_channels:".dimmed(), active_channels);
        println!("  {:<28} {}", "known_nodes:".dimmed(), known_nodes);

        println!();

        Ok(())
    }
}
