use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::{self, channel, HardwareModel};
use serde::Serialize;

use super::{Command, CommandContext};

pub struct InfoCommand;

#[derive(Serialize)]
struct InfoJson {
    node_id: String,
    num: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    long_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hw_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    is_licensed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    firmware_version: Option<String>,
    reboot_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    battery_level: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    voltage: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel_utilization: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    air_util_tx: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uptime_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    longitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    altitude: Option<i32>,
    channels: Vec<ChannelInfoJson>,
    nodes_in_mesh: usize,
}

#[derive(Serialize)]
struct ChannelInfoJson {
    index: i32,
    name: String,
    role: String,
    encryption: String,
}

#[async_trait]
impl Command for InfoCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let node_db = &ctx.node_db;
        let my_info = node_db.my_node_info();
        let local_node = node_db.local_node();
        let metadata = node_db.metadata();

        if ctx.json {
            let user = local_node.and_then(|n| n.user.as_ref());
            let metrics = local_node.and_then(|n| n.device_metrics.as_ref());
            let pos = local_node.and_then(|n| n.position.as_ref());

            let (lat, lon, alt) = pos
                .map(|p| {
                    let la = p.latitude_i.unwrap_or(0) as f64 / 1e7;
                    let lo = p.longitude_i.unwrap_or(0) as f64 / 1e7;
                    if la == 0.0 && lo == 0.0 {
                        (None, None, None)
                    } else {
                        (Some(la), Some(lo), p.altitude)
                    }
                })
                .unwrap_or((None, None, None));

            let channels: Vec<ChannelInfoJson> = node_db
                .channels()
                .iter()
                .filter(|c| c.role != channel::Role::Disabled as i32)
                .map(|ch| {
                    let settings = ch.settings.as_ref();
                    ChannelInfoJson {
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
                            .map(|s| match s.psk.len() {
                                0 => "None",
                                1 => "Default key",
                                16 => "AES-128",
                                32 => "AES-256",
                                _ => "Custom",
                            })
                            .unwrap_or("Unknown")
                            .to_string(),
                    }
                })
                .collect();

            let info = InfoJson {
                node_id: format!("!{:08x}", my_info.my_node_num),
                num: my_info.my_node_num,
                long_name: user.map(|u| u.long_name.clone()),
                short_name: user.map(|u| u.short_name.clone()),
                hw_model: user.map(|u| format_hardware(u.hw_model)),
                role: user.map(|u| format_role(u.role)),
                is_licensed: user.is_some_and(|u| u.is_licensed),
                firmware_version: metadata.map(|m| m.firmware_version.clone()),
                reboot_count: my_info.reboot_count,
                battery_level: metrics.and_then(|m| m.battery_level),
                voltage: metrics.and_then(|m| m.voltage),
                channel_utilization: metrics.and_then(|m| m.channel_utilization),
                air_util_tx: metrics.and_then(|m| m.air_util_tx),
                uptime_seconds: metrics.and_then(|m| m.uptime_seconds),
                latitude: lat,
                longitude: lon,
                altitude: alt,
                channels,
                nodes_in_mesh: node_db.nodes().len(),
            };

            println!("{}", serde_json::to_string_pretty(&info)?);
            return Ok(());
        }

        print_section("Node");
        print_field("ID", &format!("!{:08x}", my_info.my_node_num));

        if let Some(user) = local_node.and_then(|n| n.user.as_ref()) {
            print_field("Name", &user.long_name);
            print_field("Short name", &user.short_name);
            print_field("Hardware", &format_hardware(user.hw_model));
            print_field("Role", &format_role(user.role));
            if user.is_licensed {
                print_field("Licensed", "Yes (HAM)");
            }
        }

        if let Some(meta) = metadata {
            println!();
            print_section("Firmware");
            print_field("Version", &meta.firmware_version);
            print_field("Reboots", &my_info.reboot_count.to_string());

            println!();
            print_section("Capabilities");
            print_capabilities(meta);
        }

        if let Some(metrics) = local_node.and_then(|n| n.device_metrics.as_ref()) {
            println!();
            print_section("Device Metrics");
            print_device_metrics(metrics);
        }

        if let Some(pos) = local_node.and_then(|n| n.position.as_ref()) {
            if pos.latitude_i.unwrap_or(0) != 0 || pos.longitude_i.unwrap_or(0) != 0 {
                println!();
                print_section("Position");
                print_position(pos);
            }
        }

        let active_channels: Vec<_> = node_db
            .channels()
            .iter()
            .filter(|c| c.role != channel::Role::Disabled as i32)
            .collect();

        if !active_channels.is_empty() {
            println!();
            print_section("Channels");
            for ch in &active_channels {
                print_channel(ch);
            }
        }

        println!();
        print_field("Nodes in mesh", &node_db.nodes().len().to_string());

        Ok(())
    }
}

fn print_section(title: &str) {
    println!("{}", title.bold().underline());
}

fn print_field(label: &str, value: &str) {
    println!("  {:<16} {}", format!("{}:", label).dimmed(), value);
}

fn format_hardware(hw_model: i32) -> String {
    HardwareModel::try_from(hw_model)
        .map(|m| m.as_str_name().replace('_', " "))
        .unwrap_or_else(|_| format!("Unknown ({})", hw_model))
}

fn format_role(role: i32) -> String {
    use meshtastic::protobufs::config::device_config::Role;
    Role::try_from(role)
        .map(|r| r.as_str_name().replace('_', " "))
        .unwrap_or_else(|_| format!("Unknown ({})", role))
}

fn print_capabilities(meta: &protobufs::DeviceMetadata) {
    let mut caps = Vec::new();
    if meta.has_wifi {
        caps.push("WiFi");
    }
    if meta.has_bluetooth {
        caps.push("Bluetooth");
    }
    if meta.has_ethernet {
        caps.push("Ethernet");
    }
    if meta.has_pkc {
        caps.push("PKC");
    }
    if meta.can_shutdown {
        caps.push("Shutdown");
    }

    if caps.is_empty() {
        print_field("Features", "None reported");
    } else {
        print_field("Features", &caps.join(", "));
    }
}

fn print_device_metrics(metrics: &protobufs::DeviceMetrics) {
    if let Some(bat) = metrics.battery_level {
        let label = if bat > 100 {
            "Powered"
        } else {
            &format!("{}%", bat)
        };
        print_field("Battery", label);
    }
    if let Some(v) = metrics.voltage {
        print_field("Voltage", &format!("{:.2}V", v));
    }
    if let Some(cu) = metrics.channel_utilization {
        print_field("Channel util.", &format!("{:.1}%", cu));
    }
    if let Some(at) = metrics.air_util_tx {
        print_field("Air util. TX", &format!("{:.1}%", at));
    }
    if let Some(up) = metrics.uptime_seconds {
        print_field("Uptime", &format_uptime(up));
    }
}

fn format_uptime(seconds: u32) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

fn print_position(pos: &protobufs::Position) {
    let lat = pos.latitude_i.unwrap_or(0) as f64 * 1e-7;
    let lon = pos.longitude_i.unwrap_or(0) as f64 * 1e-7;
    print_field("Latitude", &format!("{:.6}", lat));
    print_field("Longitude", &format!("{:.6}", lon));
    if let Some(alt) = pos.altitude {
        if alt != 0 {
            print_field("Altitude", &format!("{}m", alt));
        }
    }
    if pos.sats_in_view > 0 {
        print_field("Satellites", &pos.sats_in_view.to_string());
    }
}

fn print_channel(ch: &protobufs::Channel) {
    let role_str = match channel::Role::try_from(ch.role) {
        Ok(channel::Role::Primary) => "Primary",
        Ok(channel::Role::Secondary) => "Secondary",
        _ => "Unknown",
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
        .unwrap_or_else(|| "Default".to_string());

    let encryption = settings
        .map(|s| match s.psk.len() {
            0 => "None",
            1 => "Default key",
            16 => "AES-128",
            32 => "AES-256",
            _ => "Custom",
        })
        .unwrap_or("Unknown");

    print_field(
        &format!("Ch {}", ch.index),
        &format!("{} ({}, {})", name, role_str, encryption),
    );
}
