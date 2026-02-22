use std::path::Path;

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use image::Luma;
use meshtastic::protobufs::{self, channel, ChannelSettings};
use meshtastic::Message;
use qrcode::render::svg;
use qrcode::QrCode;
use serde::Serialize;

use super::parsers::parse_bool;
use super::{Command, CommandContext};

#[derive(Serialize)]
struct ChannelListJson {
    index: i32,
    name: String,
    role: String,
    encryption: String,
    uplink_enabled: bool,
    downlink_enabled: bool,
}

#[derive(Serialize)]
struct ChannelQrJson {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_index: Option<i32>,
    url: String,
}

// ── ChannelListCommand ─────────────────────────────────────────────

pub struct ChannelListCommand {
    pub json: bool,
}

#[async_trait]
impl Command for ChannelListCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let channels = ctx.node_db.channels();

        if channels.is_empty() {
            if self.json {
                println!("[]");
            } else {
                println!("{}", "(no channels configured)".dimmed());
            }
            return Ok(());
        }

        if self.json {
            let json_channels: Vec<ChannelListJson> = channels
                .iter()
                .filter(|ch| ch.role != channel::Role::Disabled as i32)
                .map(|ch| {
                    let settings = ch.settings.as_ref();
                    ChannelListJson {
                        index: ch.index,
                        name: settings
                            .map(|s| {
                                if s.name.is_empty() {
                                    "Default".to_string()
                                } else {
                                    s.name.clone()
                                }
                            })
                            .unwrap_or_else(|| "Default".to_string()),
                        role: match channel::Role::try_from(ch.role) {
                            Ok(channel::Role::Primary) => "Primary".to_string(),
                            Ok(channel::Role::Secondary) => "Secondary".to_string(),
                            _ => "Unknown".to_string(),
                        },
                        encryption: settings
                            .map(|s| format_psk(&s.psk))
                            .unwrap_or_else(|| "Unknown".to_string()),
                        uplink_enabled: settings.is_some_and(|s| s.uplink_enabled),
                        downlink_enabled: settings.is_some_and(|s| s.downlink_enabled),
                    }
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&json_channels)?);
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
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
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
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
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
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
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

// ── ChannelQrCommand ──────────────────────────────────────────────

pub struct ChannelQrCommand {
    pub output: Option<String>,
    pub all: bool,
    pub json: bool,
}

#[async_trait]
impl Command for ChannelQrCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let channels = ctx.node_db.channels();
        let lora_config = ctx.node_db.local_config().lora.clone();

        let active_channels: Vec<&protobufs::Channel> = channels
            .iter()
            .filter(|ch| ch.role != channel::Role::Disabled as i32)
            .collect();

        if active_channels.is_empty() {
            println!("{}", "(no enabled channels to share)".dimmed());
            return Ok(());
        }

        if self.all {
            if self.output.is_some() {
                bail!("--all and --output cannot be used together");
            }

            if self.json {
                let mut results = Vec::new();
                for ch in &active_channels {
                    let Some(settings) = ch.settings.clone() else {
                        continue;
                    };
                    let name = if settings.name.is_empty() {
                        "Default".to_string()
                    } else {
                        settings.name.clone()
                    };
                    let channel_set = protobufs::ChannelSet {
                        settings: vec![settings],
                        lora_config: lora_config.clone(),
                    };
                    let encoded = channel_set.encode_to_vec();
                    let b64 = base64_url_encode(&encoded);
                    let url = format!("https://meshtastic.org/e/#{}", b64);
                    results.push(ChannelQrJson {
                        channel_name: Some(name),
                        channel_index: Some(ch.index),
                        url,
                    });
                }
                println!("{}", serde_json::to_string_pretty(&results)?);
                return Ok(());
            }

            for ch in &active_channels {
                let Some(settings) = ch.settings.clone() else {
                    continue;
                };
                let name = if settings.name.is_empty() {
                    "Default".to_string()
                } else {
                    settings.name.clone()
                };

                let channel_set = protobufs::ChannelSet {
                    settings: vec![settings],
                    lora_config: lora_config.clone(),
                };

                let encoded = channel_set.encode_to_vec();
                let b64 = base64_url_encode(&encoded);
                let url = format!("https://meshtastic.org/e/#{}", b64);

                println!("{} {} (index {})", "Channel:".bold(), name.bold(), ch.index);

                let code = QrCode::new(url.as_bytes())?;
                render_terminal_qr(&code);
                println!("{} {}", "URL:".bold(), url);
                println!();
            }
        } else {
            let settings: Vec<ChannelSettings> = active_channels
                .iter()
                .filter_map(|ch| ch.settings.clone())
                .collect();

            let channel_set = protobufs::ChannelSet {
                settings,
                lora_config,
            };

            let encoded = channel_set.encode_to_vec();
            let b64 = base64_url_encode(&encoded);
            let url = format!("https://meshtastic.org/e/#{}", b64);

            if self.json {
                let result = ChannelQrJson {
                    channel_name: None,
                    channel_index: None,
                    url,
                };
                println!("{}", serde_json::to_string_pretty(&result)?);
                return Ok(());
            }

            let code = QrCode::new(url.as_bytes())?;

            match &self.output {
                Some(path) => {
                    let ext = Path::new(path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();

                    match ext.as_str() {
                        "png" => {
                            let img = code.render::<Luma<u8>>().min_dimensions(512, 512).build();
                            img.save(path)?;
                            println!("{} QR code saved to {}", "ok".green(), path.bold());
                        }
                        "svg" => {
                            let svg_xml =
                                code.render::<svg::Color>().min_dimensions(256, 256).build();
                            std::fs::write(path, svg_xml)?;
                            println!("{} QR code saved to {}", "ok".green(), path.bold());
                        }
                        _ => bail!("Unsupported format '.{}'. Use .png or .svg", ext),
                    }
                }
                None => {
                    render_terminal_qr(&code);
                }
            }

            println!();
            println!("{} {}", "URL:".bold(), url);
        }

        Ok(())
    }
}

fn render_terminal_qr(code: &QrCode) {
    let width = code.width();
    let data = code.to_colors();

    // Top quiet zone
    let line_width = width + 4; // 2 module quiet zone on each side
    let empty_line: String = std::iter::repeat_n("\u{2588}\u{2588}", line_width).collect();
    println!("{}", empty_line);
    println!("{}", empty_line);

    for y in 0..width {
        // Left quiet zone
        print!("\u{2588}\u{2588}\u{2588}\u{2588}");
        for x in 0..width {
            let idx = y * width + x;
            match data[idx] {
                qrcode::Color::Dark => print!("  "), // dark module
                qrcode::Color::Light => print!("\u{2588}\u{2588}"), // light module
            }
        }
        // Right quiet zone
        println!("\u{2588}\u{2588}\u{2588}\u{2588}");
    }

    // Bottom quiet zone
    println!("{}", empty_line);
    println!("{}", empty_line);
}

// ── Helpers ────────────────────────────────────────────────────────

pub fn find_next_free_index(channels: &[protobufs::Channel]) -> anyhow::Result<i32> {
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
            let bytes = super::parsers::hex_decode(hex)?;
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

fn rand_byte() -> u8 {
    let id: u32 = meshtastic::utils::generate_rand_id();
    (id & 0xFF) as u8
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

fn base64_url_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let chunks = bytes.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;

        result.push(TABLE[((n >> 18) & 0x3F) as usize] as char);
        result.push(TABLE[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(TABLE[((n >> 6) & 0x3F) as usize] as char);
        }
        if chunk.len() > 2 {
            result.push(TABLE[(n & 0x3F) as usize] as char);
        }
    }

    result
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
