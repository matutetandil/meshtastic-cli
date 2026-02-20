use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::{self, channel, ChannelSettings};

use super::{Command, CommandContext};

// ── ChannelListCommand ─────────────────────────────────────────────

pub struct ChannelListCommand;

#[async_trait]
impl Command for ChannelListCommand {
    async fn execute(self: Box<Self>, ctx: CommandContext) -> anyhow::Result<()> {
        let channels = ctx.node_db.channels();

        if channels.is_empty() {
            println!("{}", "(no channels configured)".dimmed());
            return Ok(());
        }

        println!("{}", "Channels".bold().underline());
        for ch in channels {
            print_channel(ch);
        }
        println!();

        Ok(())
    }
}

// ── ChannelAddCommand ──────────────────────────────────────────────

pub struct ChannelAddCommand {
    pub name: String,
    pub psk: String,
}

#[async_trait]
impl Command for ChannelAddCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        if self.name.len() > 11 {
            bail!(
                "Channel name must be 11 characters or less, got {}",
                self.name.len()
            );
        }

        let channels = ctx.node_db.channels();
        let next_index = find_next_free_index(channels)?;

        let psk = parse_psk(&self.psk)?;

        let channel = protobufs::Channel {
            index: next_index,
            role: channel::Role::Secondary as i32,
            settings: Some(ChannelSettings {
                name: self.name.clone(),
                psk,
                ..Default::default()
            }),
        };

        println!(
            "{} Adding channel {} at index {}...",
            "->".cyan(),
            self.name.bold(),
            next_index
        );

        ctx.api
            .update_channel_config(&mut ctx.router, channel)
            .await?;

        println!(
            "{} Channel '{}' added at index {}.",
            "ok".green(),
            self.name,
            next_index
        );

        Ok(())
    }
}

// ── ChannelDelCommand ──────────────────────────────────────────────

pub struct ChannelDelCommand {
    pub index: u32,
}

#[async_trait]
impl Command for ChannelDelCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        if self.index == 0 {
            bail!(
                "Cannot delete primary channel (index 0). Use 'channel set' to modify it instead."
            );
        }
        if self.index > 7 {
            bail!("Channel index must be 0-7, got {}", self.index);
        }

        let channels = ctx.node_db.channels();
        let existing = channels.iter().find(|c| c.index == self.index as i32);

        let label = existing
            .and_then(|c| c.settings.as_ref())
            .map(|s| {
                if s.name.is_empty() {
                    format!("channel {}", self.index)
                } else {
                    s.name.clone()
                }
            })
            .unwrap_or_else(|| format!("channel {}", self.index));

        let channel = protobufs::Channel {
            index: self.index as i32,
            role: channel::Role::Disabled as i32,
            settings: Some(ChannelSettings::default()),
        };

        println!("{} Deleting {}...", "->".cyan(), label.bold());

        ctx.api
            .update_channel_config(&mut ctx.router, channel)
            .await?;

        println!("{} Channel {} deleted.", "ok".green(), self.index);

        Ok(())
    }
}

// ── ChannelSetCommand ──────────────────────────────────────────────

pub struct ChannelSetCommand {
    pub index: u32,
    pub field: String,
    pub value: String,
}

#[async_trait]
impl Command for ChannelSetCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        if self.index > 7 {
            bail!("Channel index must be 0-7, got {}", self.index);
        }

        let channels = ctx.node_db.channels();
        let existing = channels
            .iter()
            .find(|c| c.index == self.index as i32)
            .cloned()
            .unwrap_or(protobufs::Channel {
                index: self.index as i32,
                role: if self.index == 0 {
                    channel::Role::Primary as i32
                } else {
                    channel::Role::Secondary as i32
                },
                settings: Some(ChannelSettings::default()),
            });

        let mut settings = existing.settings.clone().unwrap_or_default();
        let mut module = settings.module_settings.unwrap_or_default();

        match self.field.as_str() {
            "name" => {
                if self.value.len() > 11 {
                    bail!("Channel name must be 11 characters or less");
                }
                settings.name = self.value.clone();
            }
            "psk" => {
                settings.psk = parse_psk(&self.value)?;
            }
            "uplink_enabled" => {
                settings.uplink_enabled = parse_bool(&self.value)?;
            }
            "downlink_enabled" => {
                settings.downlink_enabled = parse_bool(&self.value)?;
            }
            "position_precision" => {
                module.position_precision = self
                    .value
                    .parse::<u32>()
                    .map_err(|_| anyhow::anyhow!("Invalid u32 value '{}'", self.value))?;
                settings.module_settings = Some(module);
            }
            _ => bail!(
                "Unknown channel field '{}'. Valid fields: name, psk, uplink_enabled, downlink_enabled, position_precision",
                self.field
            ),
        }

        let channel = protobufs::Channel {
            index: existing.index,
            role: existing.role,
            settings: Some(settings),
        };

        println!(
            "{} Setting channel {}.{} = {}",
            "->".cyan(),
            self.index,
            self.field,
            self.value.bold()
        );

        ctx.api
            .update_channel_config(&mut ctx.router, channel)
            .await?;

        println!("{} Channel {} updated.", "ok".green(), self.index);

        Ok(())
    }
}

// ── Helpers ────────────────────────────────────────────────────────

fn find_next_free_index(channels: &[protobufs::Channel]) -> anyhow::Result<i32> {
    for i in 1..=7 {
        let is_used = channels
            .iter()
            .any(|c| c.index == i && c.role != channel::Role::Disabled as i32);
        if !is_used {
            return Ok(i);
        }
    }
    bail!("No free channel slots available (max 8 channels, indices 0-7)")
}

fn parse_psk(value: &str) -> anyhow::Result<Vec<u8>> {
    match value.to_lowercase().as_str() {
        "none" => Ok(vec![]),
        "default" => Ok(vec![1]),
        "random" => {
            let mut key = vec![0u8; 32];
            for byte in &mut key {
                *byte = rand_byte();
            }
            Ok(key)
        }
        _ => {
            let hex = value.strip_prefix("0x").unwrap_or(value);
            let bytes = hex_decode(hex)?;
            match bytes.len() {
                16 | 32 => Ok(bytes),
                _ => bail!(
                    "PSK must be 16 bytes (32 hex chars) for AES-128 or 32 bytes (64 hex chars) for AES-256. Got {} bytes.",
                    bytes.len()
                ),
            }
        }
    }
}

fn hex_decode(hex: &str) -> anyhow::Result<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        bail!("Hex string must have even length");
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| anyhow::anyhow!("Invalid hex character in '{}'", &hex[i..i + 2]))
        })
        .collect()
}

fn rand_byte() -> u8 {
    let id: u32 = meshtastic::utils::generate_rand_id();
    (id & 0xFF) as u8
}

fn parse_bool(value: &str) -> anyhow::Result<bool> {
    match value.to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => Ok(true),
        "false" | "0" | "no" | "off" => Ok(false),
        _ => bail!(
            "Invalid boolean '{}'. Use true/false, 1/0, yes/no, or on/off.",
            value
        ),
    }
}

fn print_channel(ch: &protobufs::Channel) {
    let role = channel::Role::try_from(ch.role);
    let role_str = match role {
        Ok(channel::Role::Primary) => "Primary".green().to_string(),
        Ok(channel::Role::Secondary) => "Secondary".cyan().to_string(),
        Ok(channel::Role::Disabled) => "Disabled".dimmed().to_string(),
        Err(_) => format!("Unknown({})", ch.role),
    };

    let settings = ch.settings.as_ref();

    let name = settings
        .map(|s| {
            if s.name.is_empty() {
                "Default".to_string()
            } else {
                s.name.clone()
            }
        })
        .unwrap_or_else(|| "(none)".to_string());

    let encryption = settings
        .map(|s| format_psk(&s.psk))
        .unwrap_or_else(|| "Unknown".to_string());

    let uplink = settings.is_some_and(|s| s.uplink_enabled);
    let downlink = settings.is_some_and(|s| s.downlink_enabled);

    println!(
        "  {:<6} {:<14} {:<12} {:<12} uplink: {:<5} downlink: {}",
        format!("[{}]", ch.index).bold(),
        name,
        role_str,
        encryption,
        uplink,
        downlink
    );
}

fn format_psk(psk: &[u8]) -> String {
    match psk.len() {
        0 => "None".to_string(),
        1 => "Default key".to_string(),
        16 => "AES-128".to_string(),
        32 => "AES-256".to_string(),
        _ => format!("Custom ({} bytes)", psk.len()),
    }
}
