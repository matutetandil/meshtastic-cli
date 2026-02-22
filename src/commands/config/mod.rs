pub(super) mod field_mapper;
mod printer;
mod serializer;

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs;
use meshtastic::Message;
use serde_json::json;

use crate::cli::ConfigSection;

use super::{Command, CommandContext};

pub(super) use field_mapper::{apply_config_field, apply_module_config_field};
use printer::*;
use serializer::*;

// ── ConfigGetCommand ───────────────────────────────────────────────

pub struct ConfigGetCommand {
    pub section: Option<ConfigSection>,
    pub json: bool,
}

#[async_trait]
impl Command for ConfigGetCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let config = ctx.node_db.local_config();
        let module = ctx.node_db.local_module_config();

        if self.json {
            let result = match &self.section {
                None => {
                    let mut map = serde_json::Map::new();
                    if let Some(c) = config.device.as_ref() {
                        map.insert("device".into(), config_device_json(c));
                    }
                    if let Some(c) = config.position.as_ref() {
                        map.insert("position".into(), config_position_json(c));
                    }
                    if let Some(c) = config.power.as_ref() {
                        map.insert("power".into(), config_power_json(c));
                    }
                    if let Some(c) = config.network.as_ref() {
                        map.insert("network".into(), config_network_json(c));
                    }
                    if let Some(c) = config.display.as_ref() {
                        map.insert("display".into(), config_display_json(c));
                    }
                    if let Some(c) = config.lora.as_ref() {
                        map.insert("lora".into(), config_lora_json(c));
                    }
                    if let Some(c) = config.bluetooth.as_ref() {
                        map.insert("bluetooth".into(), config_bluetooth_json(c));
                    }
                    if let Some(c) = config.security.as_ref() {
                        map.insert("security".into(), config_security_json(c));
                    }
                    if let Some(c) = module.mqtt.as_ref() {
                        map.insert("mqtt".into(), config_mqtt_json(c));
                    }
                    if let Some(c) = module.serial.as_ref() {
                        map.insert("serial".into(), config_serial_json(c));
                    }
                    if let Some(c) = module.telemetry.as_ref() {
                        map.insert("telemetry".into(), config_telemetry_json(c));
                    }
                    if let Some(c) = module.neighbor_info.as_ref() {
                        map.insert("neighbor_info".into(), config_neighbor_info_json(c));
                    }
                    serde_json::Value::Object(map)
                }
                Some(section) => {
                    let val = match section {
                        ConfigSection::Device => config.device.as_ref().map(config_device_json),
                        ConfigSection::Position => {
                            config.position.as_ref().map(config_position_json)
                        }
                        ConfigSection::Power => config.power.as_ref().map(config_power_json),
                        ConfigSection::Network => config.network.as_ref().map(config_network_json),
                        ConfigSection::Display => config.display.as_ref().map(config_display_json),
                        ConfigSection::Lora => config.lora.as_ref().map(config_lora_json),
                        ConfigSection::Bluetooth => {
                            config.bluetooth.as_ref().map(config_bluetooth_json)
                        }
                        ConfigSection::Security => {
                            config.security.as_ref().map(config_security_json)
                        }
                        ConfigSection::Mqtt => module.mqtt.as_ref().map(config_mqtt_json),
                        ConfigSection::Serial => module.serial.as_ref().map(config_serial_json),
                        ConfigSection::Telemetry => {
                            module.telemetry.as_ref().map(config_telemetry_json)
                        }
                        ConfigSection::NeighborInfo => {
                            module.neighbor_info.as_ref().map(config_neighbor_info_json)
                        }
                        _ => Some(json!({"status": "section not available in JSON mode"})),
                    };
                    val.unwrap_or(serde_json::Value::Null)
                }
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }

        match &self.section {
            None => {
                print_device(config.device.as_ref());
                print_position(config.position.as_ref());
                print_power(config.power.as_ref());
                print_network(config.network.as_ref());
                print_display(config.display.as_ref());
                print_lora(config.lora.as_ref());
                print_bluetooth(config.bluetooth.as_ref());
                print_security(config.security.as_ref());
                print_mqtt(module.mqtt.as_ref());
                print_serial(module.serial.as_ref());
                print_external_notification(module.external_notification.as_ref());
                print_store_forward(module.store_forward.as_ref());
                print_range_test(module.range_test.as_ref());
                print_telemetry(module.telemetry.as_ref());
                print_canned_message(module.canned_message.as_ref());
                print_audio(module.audio.as_ref());
                print_remote_hardware(module.remote_hardware.as_ref());
                print_neighbor_info(module.neighbor_info.as_ref());
                print_ambient_lighting(module.ambient_lighting.as_ref());
                print_detection_sensor(module.detection_sensor.as_ref());
                print_paxcounter(module.paxcounter.as_ref());
            }
            Some(section) => match section {
                ConfigSection::Device => print_device(config.device.as_ref()),
                ConfigSection::Position => print_position(config.position.as_ref()),
                ConfigSection::Power => print_power(config.power.as_ref()),
                ConfigSection::Network => print_network(config.network.as_ref()),
                ConfigSection::Display => print_display(config.display.as_ref()),
                ConfigSection::Lora => print_lora(config.lora.as_ref()),
                ConfigSection::Bluetooth => print_bluetooth(config.bluetooth.as_ref()),
                ConfigSection::Security => print_security(config.security.as_ref()),
                ConfigSection::Mqtt => print_mqtt(module.mqtt.as_ref()),
                ConfigSection::Serial => print_serial(module.serial.as_ref()),
                ConfigSection::ExternalNotification => {
                    print_external_notification(module.external_notification.as_ref());
                }
                ConfigSection::StoreForward => {
                    print_store_forward(module.store_forward.as_ref());
                }
                ConfigSection::RangeTest => print_range_test(module.range_test.as_ref()),
                ConfigSection::Telemetry => print_telemetry(module.telemetry.as_ref()),
                ConfigSection::CannedMessage => {
                    print_canned_message(module.canned_message.as_ref());
                }
                ConfigSection::Audio => print_audio(module.audio.as_ref()),
                ConfigSection::RemoteHardware => {
                    print_remote_hardware(module.remote_hardware.as_ref());
                }
                ConfigSection::NeighborInfo => {
                    print_neighbor_info(module.neighbor_info.as_ref());
                }
                ConfigSection::AmbientLighting => {
                    print_ambient_lighting(module.ambient_lighting.as_ref());
                }
                ConfigSection::DetectionSensor => {
                    print_detection_sensor(module.detection_sensor.as_ref());
                }
                ConfigSection::Paxcounter => print_paxcounter(module.paxcounter.as_ref()),
            },
        }

        Ok(())
    }
}

// ── ConfigSetCommand ───────────────────────────────────────────────

pub struct ConfigSetCommand {
    pub key: String,
    pub value: String,
}

#[async_trait]
impl Command for ConfigSetCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (section, field) = self.key.split_once('.').ok_or_else(|| {
            anyhow::anyhow!("Key must be in section.field format (e.g. lora.region)")
        })?;

        println!(
            "{} Setting {}.{} = {}",
            "->".cyan(),
            section,
            field,
            self.value.bold()
        );

        let config = ctx.node_db.local_config().clone();
        let module = ctx.node_db.local_module_config().clone();

        match section {
            "device" | "position" | "power" | "network" | "display" | "lora" | "bluetooth"
            | "security" => {
                let payload = apply_config_field(section, field, &self.value, &config)?;
                let config_packet = protobufs::Config {
                    payload_variant: Some(payload),
                };
                println!(
                    "{} Device will reboot to apply changes.",
                    "!".yellow().bold()
                );
                ctx.api
                    .update_config(&mut ctx.router, config_packet)
                    .await?;
            }
            "mqtt"
            | "serial"
            | "external_notification"
            | "store_forward"
            | "range_test"
            | "telemetry"
            | "canned_message"
            | "audio"
            | "remote_hardware"
            | "neighbor_info"
            | "ambient_lighting"
            | "detection_sensor"
            | "paxcounter" => {
                let payload = apply_module_config_field(section, field, &self.value, &module)?;
                let module_packet = protobufs::ModuleConfig {
                    payload_variant: Some(payload),
                };
                println!(
                    "{} Device will reboot to apply changes.",
                    "!".yellow().bold()
                );
                ctx.api
                    .update_module_config(&mut ctx.router, module_packet)
                    .await?;
            }
            _ => bail!(
                "Unknown config section '{}'. Use 'config get' to see valid sections.",
                section
            ),
        }

        println!("{} Configuration updated.", "ok".green());

        Ok(())
    }
}

// ── SetHamCommand ─────────────────────────────────────────────────

pub struct SetHamCommand {
    pub call_sign: String,
    pub short_name: Option<String>,
    pub tx_power: Option<i32>,
    pub frequency: Option<f32>,
}

#[async_trait]
impl Command for SetHamCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let short_name = self
            .short_name
            .clone()
            .unwrap_or_else(|| self.call_sign.chars().take(4).collect::<String>());

        let ham = protobufs::HamParameters {
            call_sign: self.call_sign.clone(),
            tx_power: self.tx_power.unwrap_or(0),
            frequency: self.frequency.unwrap_or(0.0),
            short_name: short_name.clone(),
        };

        println!(
            "{} Setting HAM mode: callsign={}, short_name={}",
            "->".cyan(),
            self.call_sign.bold(),
            short_name.bold()
        );

        if let Some(power) = self.tx_power {
            println!("  tx_power: {} dBm", power);
        }
        if let Some(freq) = self.frequency {
            println!("  frequency: {} MHz", freq);
        }

        let my_id = ctx.node_db.my_node_num();
        super::admin::send_admin_message(
            ctx,
            my_id,
            protobufs::admin_message::PayloadVariant::SetHamMode(ham),
        )
        .await?;

        println!(
            "{} HAM mode configured. Device is now licensed as {}.",
            "ok".green(),
            self.call_sign
        );

        Ok(())
    }
}

// ── SetUrlCommand ─────────────────────────────────────────────────

pub struct SetUrlCommand {
    pub url: String,
}

#[async_trait]
impl Command for SetUrlCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let encoded = extract_url_payload(&self.url)?;
        let bytes = base64_decode(&encoded)?;
        let channel_set = protobufs::ChannelSet::decode(bytes.as_slice())
            .map_err(|e| anyhow::anyhow!("Failed to decode channel set from URL: {}", e))?;

        println!("{} Applying configuration from URL...", "->".cyan());

        // Apply channels
        let mut channel_count = 0u32;
        for (i, settings) in channel_set.settings.iter().enumerate() {
            let role = if i == 0 {
                protobufs::channel::Role::Primary as i32
            } else {
                protobufs::channel::Role::Secondary as i32
            };

            let channel = protobufs::Channel {
                index: i as i32,
                role,
                settings: Some(protobufs::ChannelSettings {
                    psk: settings.psk.clone(),
                    name: settings.name.clone(),
                    id: settings.id,
                    uplink_enabled: settings.uplink_enabled,
                    downlink_enabled: settings.downlink_enabled,
                    module_settings: settings.module_settings,
                    ..Default::default()
                }),
            };

            ctx.api
                .update_channel_config(&mut ctx.router, channel)
                .await?;
            channel_count += 1;

            let label = if settings.name.is_empty() {
                "Default".to_string()
            } else {
                settings.name.clone()
            };
            println!("  {} Channel {}: {}", "ok".green(), i, label);
        }

        // Apply LoRa config if present
        if let Some(lora) = channel_set.lora_config {
            let config_packet = protobufs::Config {
                payload_variant: Some(protobufs::config::PayloadVariant::Lora(lora)),
            };
            ctx.api
                .update_config(&mut ctx.router, config_packet)
                .await?;
            println!("  {} LoRa configuration applied.", "ok".green());
        }

        println!(
            "{} Applied {} channels from URL.",
            "ok".green(),
            channel_count
        );
        println!(
            "{} Device will reboot to apply changes.",
            "!".yellow().bold()
        );

        Ok(())
    }
}

// ── BeginEditCommand ──────────────────────────────────────────────

pub struct BeginEditCommand;

#[async_trait]
impl Command for BeginEditCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();

        println!("{} Starting batch edit session...", "->".cyan());

        super::admin::send_admin_message(
            ctx,
            my_id,
            protobufs::admin_message::PayloadVariant::BeginEditSettings(true),
        )
        .await?;

        println!(
            "{} Batch edit session started. Changes will be queued until 'config commit-edit'.",
            "ok".green()
        );

        Ok(())
    }
}

// ── CommitEditCommand ────────────────────────────────────────────

pub struct CommitEditCommand;

#[async_trait]
impl Command for CommitEditCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let my_id = ctx.node_db.my_node_num();

        println!("{} Committing queued configuration changes...", "->".cyan());

        super::admin::send_admin_message(
            ctx,
            my_id,
            protobufs::admin_message::PayloadVariant::CommitEditSettings(true),
        )
        .await?;

        println!("{} Configuration changes committed.", "ok".green());
        println!(
            "{} Device will reboot to apply changes.",
            "!".yellow().bold()
        );

        Ok(())
    }
}

// ── SetModemPresetCommand ────────────────────────────────────────

pub struct SetModemPresetCommand {
    pub preset: i32,
}

#[async_trait]
impl Command for SetModemPresetCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let mut lora = ctx.node_db.local_config().lora.clone().unwrap_or_default();
        lora.modem_preset = self.preset;
        lora.use_preset = true;

        let preset_name = protobufs::config::lo_ra_config::ModemPreset::try_from(self.preset)
            .map(|p| format!("{:?}", p))
            .unwrap_or_else(|_| self.preset.to_string());

        println!(
            "{} Setting modem preset to {}...",
            "->".cyan(),
            preset_name.bold()
        );

        let config_packet = protobufs::Config {
            payload_variant: Some(protobufs::config::PayloadVariant::Lora(lora)),
        };

        ctx.api
            .update_config(&mut ctx.router, config_packet)
            .await?;

        println!("{} Modem preset updated.", "ok".green());
        println!(
            "{} Device will reboot to apply changes.",
            "!".yellow().bold()
        );

        Ok(())
    }
}

// ── ChAddUrlCommand ──────────────────────────────────────────────

pub struct ChAddUrlCommand {
    pub url: String,
}

#[async_trait]
impl Command for ChAddUrlCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let encoded = extract_url_payload(&self.url)?;
        let bytes = base64_decode(&encoded)?;
        let channel_set = protobufs::ChannelSet::decode(bytes.as_slice())
            .map_err(|e| anyhow::anyhow!("Failed to decode channel set from URL: {}", e))?;

        println!(
            "{} Adding channels from URL (without replacing existing)...",
            "->".cyan()
        );

        let channels = ctx.node_db.channels();
        let mut added = 0u32;

        for settings in &channel_set.settings {
            let next_index = match super::channel::find_next_free_index(channels) {
                Ok(idx) => idx,
                Err(_) => {
                    println!(
                        "  {} No free channel slots remaining, skipping rest.",
                        "!".yellow()
                    );
                    break;
                }
            };

            let label = if settings.name.is_empty() {
                "Default".to_string()
            } else {
                settings.name.clone()
            };

            let channel = protobufs::Channel {
                index: next_index,
                role: protobufs::channel::Role::Secondary as i32,
                settings: Some(protobufs::ChannelSettings {
                    psk: settings.psk.clone(),
                    name: settings.name.clone(),
                    id: settings.id,
                    uplink_enabled: settings.uplink_enabled,
                    downlink_enabled: settings.downlink_enabled,
                    module_settings: settings.module_settings,
                    ..Default::default()
                }),
            };

            ctx.api
                .update_channel_config(&mut ctx.router, channel)
                .await?;
            added += 1;
            println!(
                "  {} Channel {}: {} (index {})",
                "ok".green(),
                added,
                label,
                next_index
            );
        }

        println!("{} Added {} channel(s) from URL.", "ok".green(), added);

        Ok(())
    }
}

fn extract_url_payload(url: &str) -> anyhow::Result<String> {
    // Support formats:
    //   https://meshtastic.org/e/#PAYLOAD
    //   meshtastic://PAYLOAD
    //   #PAYLOAD
    //   PAYLOAD (raw base64)
    if let Some(payload) = url.strip_prefix("https://meshtastic.org/e/#") {
        Ok(payload.to_string())
    } else if let Some(payload) = url.strip_prefix("http://meshtastic.org/e/#") {
        Ok(payload.to_string())
    } else if let Some(payload) = url.strip_prefix("meshtastic://") {
        Ok(payload.to_string())
    } else if let Some(payload) = url.strip_prefix('#') {
        Ok(payload.to_string())
    } else {
        // Assume raw base64
        Ok(url.to_string())
    }
}

fn base64_decode(input: &str) -> anyhow::Result<Vec<u8>> {
    // Meshtastic URLs use URL-safe base64 (no padding)
    let input = input.replace('-', "+").replace('_', "/");

    // Add padding if needed
    let padded = match input.len() % 4 {
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => input,
    };

    // Simple base64 decoder
    let mut result = Vec::new();
    let chars: Vec<u8> = padded.bytes().collect();

    for chunk in chars.chunks(4) {
        if chunk.len() < 4 {
            break;
        }

        let a = b64_val(chunk[0])?;
        let b = b64_val(chunk[1])?;
        let c = b64_val(chunk[2])?;
        let d = b64_val(chunk[3])?;

        result.push((a << 2) | (b >> 4));
        if chunk[2] != b'=' {
            result.push(((b & 0x0F) << 4) | (c >> 2));
        }
        if chunk[3] != b'=' {
            result.push(((c & 0x03) << 6) | d);
        }
    }

    Ok(result)
}

fn b64_val(c: u8) -> anyhow::Result<u8> {
    match c {
        b'A'..=b'Z' => Ok(c - b'A'),
        b'a'..=b'z' => Ok(c - b'a' + 26),
        b'0'..=b'9' => Ok(c - b'0' + 52),
        b'+' => Ok(62),
        b'/' => Ok(63),
        b'=' => Ok(0),
        _ => bail!("Invalid base64 character: {}", c as char),
    }
}
