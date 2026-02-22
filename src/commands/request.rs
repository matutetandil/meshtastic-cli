use std::time::{Duration, Instant};

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::{
    self, admin_message, telemetry, AdminMessage, Data, HardwareModel, MeshPacket, PortNum,
    Position, Telemetry,
};
use meshtastic::utils::generate_rand_id;
use meshtastic::Message;

use serde_json::json;

use super::{resolve_destination, Command, CommandContext, DestinationSpec};

// ── Telemetry type selection ──────────────────────────────────────

pub enum TelemetryType {
    Device,
    Environment,
    AirQuality,
    Power,
    LocalStats,
    Health,
    Host,
}

// ── RequestTelemetryCommand ───────────────────────────────────────

pub struct RequestTelemetryCommand {
    pub destination: DestinationSpec,
    pub timeout_secs: u64,
    pub telemetry_type: TelemetryType,
}

#[async_trait]
impl Command for RequestTelemetryCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        let target_id = match packet_dest {
            meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
            _ => bail!("Request requires a specific node destination"),
        };

        let packet_id = generate_rand_id();
        let my_node = ctx.node_db.my_node_num();

        let variant = match self.telemetry_type {
            TelemetryType::Device => Some(telemetry::Variant::DeviceMetrics(Default::default())),
            TelemetryType::Environment => {
                Some(telemetry::Variant::EnvironmentMetrics(Default::default()))
            }
            TelemetryType::AirQuality => {
                Some(telemetry::Variant::AirQualityMetrics(Default::default()))
            }
            TelemetryType::Power => Some(telemetry::Variant::PowerMetrics(Default::default())),
            TelemetryType::LocalStats => Some(telemetry::Variant::LocalStats(Default::default())),
            TelemetryType::Health => Some(telemetry::Variant::HealthMetrics(Default::default())),
            TelemetryType::Host => Some(telemetry::Variant::HostMetrics(Default::default())),
        };

        let telemetry = Telemetry { time: 0, variant };

        let mesh_packet = MeshPacket {
            from: my_node,
            to: target_id,
            id: packet_id,
            want_ack: true,
            channel: 0,
            hop_limit: 3,
            payload_variant: Some(MeshPayload::Decoded(Data {
                portnum: PortNum::TelemetryApp as i32,
                payload: telemetry.encode_to_vec(),
                want_response: true,
                ..Default::default()
            })),
            ..Default::default()
        };

        println!(
            "{} Requesting telemetry from {}...",
            "->".cyan(),
            dest_label.bold()
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
                    "{} Timeout after {}s — no telemetry from {}",
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
                        "{} Timeout after {}s — no telemetry from {}",
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
                    if data.portnum != PortNum::TelemetryApp as i32 || mp.from != target_id {
                        continue;
                    }

                    if let Ok(telem) = Telemetry::decode(data.payload.as_slice()) {
                        let elapsed = start.elapsed().as_secs_f64();
                        if ctx.json {
                            let val = telemetry_to_json(&telem, &dest_label, elapsed);
                            println!("{}", serde_json::to_string_pretty(&val)?);
                        } else {
                            println!(
                                "{} Telemetry from {} (in {:.1}s):",
                                "ok".green(),
                                dest_label,
                                elapsed
                            );
                            print_telemetry(&telem);
                        }
                        return Ok(());
                    }
                }
            }
        }
    }
}

// ── RequestPositionCommand ────────────────────────────────────────

pub struct RequestPositionCommand {
    pub destination: DestinationSpec,
    pub timeout_secs: u64,
}

#[async_trait]
impl Command for RequestPositionCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        let target_id = match packet_dest {
            meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
            _ => bail!("Request requires a specific node destination"),
        };

        let packet_id = generate_rand_id();
        let my_node = ctx.node_db.my_node_num();

        let position = Position::default();

        let mesh_packet = MeshPacket {
            from: my_node,
            to: target_id,
            id: packet_id,
            want_ack: true,
            channel: 0,
            hop_limit: 3,
            payload_variant: Some(MeshPayload::Decoded(Data {
                portnum: PortNum::PositionApp as i32,
                payload: position.encode_to_vec(),
                want_response: true,
                ..Default::default()
            })),
            ..Default::default()
        };

        println!(
            "{} Requesting position from {}...",
            "->".cyan(),
            dest_label.bold()
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
                    "{} Timeout after {}s — no position from {}",
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
                        "{} Timeout after {}s — no position from {}",
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
                    if data.portnum != PortNum::PositionApp as i32 || mp.from != target_id {
                        continue;
                    }

                    if let Ok(pos) = Position::decode(data.payload.as_slice()) {
                        let elapsed = start.elapsed().as_secs_f64();
                        if ctx.json {
                            let lat = pos.latitude_i.unwrap_or(0) as f64 / 1e7;
                            let lon = pos.longitude_i.unwrap_or(0) as f64 / 1e7;
                            let val = json!({
                                "source": dest_label,
                                "rtt_s": elapsed,
                                "latitude": lat,
                                "longitude": lon,
                                "altitude": pos.altitude.unwrap_or(0),
                                "sats_in_view": pos.sats_in_view,
                            });
                            println!("{}", serde_json::to_string_pretty(&val)?);
                        } else {
                            println!(
                                "{} Position from {} (in {:.1}s):",
                                "ok".green(),
                                dest_label,
                                elapsed
                            );
                            print_position(&pos);
                        }
                        return Ok(());
                    }
                }
            }
        }
    }
}

// ── RequestMetadataCommand ───────────────────────────────────────

pub struct RequestMetadataCommand {
    pub destination: DestinationSpec,
    pub timeout_secs: u64,
}

#[async_trait]
impl Command for RequestMetadataCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        let target_id = match packet_dest {
            meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
            _ => bail!("Request requires a specific node destination"),
        };

        let packet_id = generate_rand_id();
        let my_node = ctx.node_db.my_node_num();

        let admin_msg = AdminMessage {
            payload_variant: Some(admin_message::PayloadVariant::GetDeviceMetadataRequest(
                true,
            )),
            session_passkey: Vec::new(),
        };

        let mesh_packet = MeshPacket {
            from: my_node,
            to: target_id,
            id: packet_id,
            want_ack: true,
            channel: 0,
            hop_limit: 3,
            payload_variant: Some(MeshPayload::Decoded(Data {
                portnum: PortNum::AdminApp as i32,
                payload: admin_msg.encode_to_vec(),
                want_response: true,
                ..Default::default()
            })),
            ..Default::default()
        };

        println!(
            "{} Requesting metadata from {}...",
            "->".cyan(),
            dest_label.bold()
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
                    "{} Timeout after {}s — no metadata from {}",
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
                        "{} Timeout after {}s — no metadata from {}",
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
                    if data.portnum != PortNum::AdminApp as i32 || mp.from != target_id {
                        continue;
                    }

                    if let Ok(admin) = AdminMessage::decode(data.payload.as_slice()) {
                        if let Some(admin_message::PayloadVariant::GetDeviceMetadataResponse(
                            meta,
                        )) = admin.payload_variant
                        {
                            let elapsed = start.elapsed().as_secs_f64();
                            if ctx.json {
                                let hw = HardwareModel::try_from(meta.hw_model)
                                    .map(|m| m.as_str_name().to_string())
                                    .unwrap_or_else(|_| format!("Unknown({})", meta.hw_model));
                                let role =
                                    protobufs::config::device_config::Role::try_from(meta.role)
                                        .map(|r| format!("{:?}", r))
                                        .unwrap_or_else(|_| meta.role.to_string());
                                let val = json!({
                                    "source": dest_label,
                                    "rtt_s": elapsed,
                                    "firmware_version": meta.firmware_version,
                                    "device_state_version": meta.device_state_version,
                                    "hw_model": hw,
                                    "role": role,
                                    "can_shutdown": meta.can_shutdown,
                                    "has_wifi": meta.has_wifi,
                                    "has_bluetooth": meta.has_bluetooth,
                                    "has_ethernet": meta.has_ethernet,
                                    "has_remote_hardware": meta.has_remote_hardware,
                                    "has_pkc": meta.has_pkc,
                                });
                                println!("{}", serde_json::to_string_pretty(&val)?);
                            } else {
                                println!(
                                    "{} Metadata from {} (in {:.1}s):",
                                    "ok".green(),
                                    dest_label,
                                    elapsed
                                );
                                print_metadata(&meta);
                            }
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

// ── Display helpers ───────────────────────────────────────────────

fn print_telemetry(telem: &Telemetry) {
    match &telem.variant {
        Some(telemetry::Variant::DeviceMetrics(m)) => {
            println!("  {}", "Device Metrics".bold());
            if let Some(b) = m.battery_level {
                if b > 0 {
                    println!("    {:<24} {}%", "battery:".dimmed(), b);
                }
            }
            if let Some(v) = m.voltage {
                if v > 0.0 {
                    println!("    {:<24} {:.2}V", "voltage:".dimmed(), v);
                }
            }
            if let Some(cu) = m.channel_utilization {
                if cu > 0.0 {
                    println!("    {:<24} {:.1}%", "channel_utilization:".dimmed(), cu);
                }
            }
            if let Some(au) = m.air_util_tx {
                if au > 0.0 {
                    println!("    {:<24} {:.1}%", "air_util_tx:".dimmed(), au);
                }
            }
            if let Some(up) = m.uptime_seconds {
                if up > 0 {
                    println!("    {:<24} {}", "uptime:".dimmed(), format_uptime(up));
                }
            }
        }
        Some(telemetry::Variant::EnvironmentMetrics(m)) => {
            println!("  {}", "Environment Metrics".bold());
            if let Some(t) = m.temperature {
                if t > 0.0 {
                    println!("    {:<24} {:.1} C", "temperature:".dimmed(), t);
                }
            }
            if let Some(h) = m.relative_humidity {
                if h > 0.0 {
                    println!("    {:<24} {:.1}%", "humidity:".dimmed(), h);
                }
            }
            if let Some(p) = m.barometric_pressure {
                if p > 0.0 {
                    println!("    {:<24} {:.1} hPa", "pressure:".dimmed(), p);
                }
            }
        }
        Some(telemetry::Variant::PowerMetrics(m)) => {
            println!("  {}", "Power Metrics".bold());
            if let Some(v) = m.ch1_voltage {
                if v > 0.0 {
                    println!("    {:<24} {:.2}V", "ch1_voltage:".dimmed(), v);
                }
            }
            if let Some(c) = m.ch1_current {
                if c > 0.0 {
                    println!("    {:<24} {:.1} mA", "ch1_current:".dimmed(), c);
                }
            }
            if let Some(v) = m.ch2_voltage {
                if v > 0.0 {
                    println!("    {:<24} {:.2}V", "ch2_voltage:".dimmed(), v);
                }
            }
            if let Some(c) = m.ch2_current {
                if c > 0.0 {
                    println!("    {:<24} {:.1} mA", "ch2_current:".dimmed(), c);
                }
            }
            if let Some(v) = m.ch3_voltage {
                if v > 0.0 {
                    println!("    {:<24} {:.2}V", "ch3_voltage:".dimmed(), v);
                }
            }
            if let Some(c) = m.ch3_current {
                if c > 0.0 {
                    println!("    {:<24} {:.1} mA", "ch3_current:".dimmed(), c);
                }
            }
        }
        Some(telemetry::Variant::AirQualityMetrics(m)) => {
            println!("  {}", "Air Quality Metrics".bold());
            if let Some(v) = m.pm10_standard {
                println!("    {:<24} {} ug/m3", "pm1.0:".dimmed(), v);
            }
            if let Some(v) = m.pm25_standard {
                println!("    {:<24} {} ug/m3", "pm2.5:".dimmed(), v);
            }
            if let Some(v) = m.pm100_standard {
                println!("    {:<24} {} ug/m3", "pm10.0:".dimmed(), v);
            }
            if let Some(v) = m.co2 {
                println!("    {:<24} {} ppm", "co2:".dimmed(), v);
            }
            if let Some(v) = m.pm_voc_idx {
                if v > 0.0 {
                    println!("    {:<24} {:.1}", "voc_index:".dimmed(), v);
                }
            }
            if let Some(v) = m.pm_nox_idx {
                if v > 0.0 {
                    println!("    {:<24} {:.1}", "nox_index:".dimmed(), v);
                }
            }
        }
        Some(telemetry::Variant::LocalStats(m)) => {
            println!("  {}", "Local Stats".bold());
            if m.uptime_seconds > 0 {
                println!(
                    "    {:<24} {}",
                    "uptime:".dimmed(),
                    format_uptime(m.uptime_seconds)
                );
            }
            println!(
                "    {:<24} {:.1}%",
                "channel_utilization:".dimmed(),
                m.channel_utilization
            );
            println!("    {:<24} {:.1}%", "air_util_tx:".dimmed(), m.air_util_tx);
            println!("    {:<24} {}", "packets_tx:".dimmed(), m.num_packets_tx);
            println!("    {:<24} {}", "packets_rx:".dimmed(), m.num_packets_rx);
            println!(
                "    {:<24} {}",
                "packets_rx_bad:".dimmed(),
                m.num_packets_rx_bad
            );
            println!("    {:<24} {}", "rx_duplicates:".dimmed(), m.num_rx_dupe);
            println!("    {:<24} {}", "tx_relayed:".dimmed(), m.num_tx_relay);
            println!(
                "    {:<24} {}",
                "online_nodes:".dimmed(),
                m.num_online_nodes
            );
            println!("    {:<24} {}", "total_nodes:".dimmed(), m.num_total_nodes);
        }
        Some(telemetry::Variant::HealthMetrics(m)) => {
            println!("  {}", "Health Metrics".bold());
            if let Some(v) = m.heart_bpm {
                println!("    {:<24} {} bpm", "heart_rate:".dimmed(), v);
            }
            if let Some(v) = m.sp_o2 {
                println!("    {:<24} {}%", "spo2:".dimmed(), v);
            }
        }
        Some(telemetry::Variant::HostMetrics(m)) => {
            println!("  {}", "Host Metrics".bold());
            if m.uptime_seconds > 0 {
                println!(
                    "    {:<24} {}",
                    "uptime:".dimmed(),
                    format_uptime(m.uptime_seconds)
                );
            }
            if m.freemem_bytes > 0 {
                println!(
                    "    {:<24} {} bytes",
                    "free_memory:".dimmed(),
                    m.freemem_bytes
                );
            }
            if m.diskfree1_bytes > 0 {
                println!(
                    "    {:<24} {} bytes",
                    "disk_free:".dimmed(),
                    m.diskfree1_bytes
                );
            }
            if m.load1 > 0 {
                println!(
                    "    {:<24} {:.2}",
                    "load_1m:".dimmed(),
                    m.load1 as f64 / 100.0
                );
            }
            if m.load5 > 0 {
                println!(
                    "    {:<24} {:.2}",
                    "load_5m:".dimmed(),
                    m.load5 as f64 / 100.0
                );
            }
            if m.load15 > 0 {
                println!(
                    "    {:<24} {:.2}",
                    "load_15m:".dimmed(),
                    m.load15 as f64 / 100.0
                );
            }
        }
        None => {
            println!("  {}", "(empty telemetry response)".dimmed());
        }
    }
}

fn print_position(pos: &Position) {
    let lat = pos.latitude_i.unwrap_or(0) as f64 / 1e7;
    let lon = pos.longitude_i.unwrap_or(0) as f64 / 1e7;
    let alt = pos.altitude.unwrap_or(0);

    println!("  {:<20} {:.7}", "latitude:".dimmed(), lat);
    println!("  {:<20} {:.7}", "longitude:".dimmed(), lon);
    println!("  {:<20} {} m", "altitude:".dimmed(), alt);
    if pos.sats_in_view > 0 {
        println!("  {:<20} {}", "sats_in_view:".dimmed(), pos.sats_in_view);
    }
    if pos.gps_accuracy > 0 {
        println!("  {:<20} {} mm", "gps_accuracy:".dimmed(), pos.gps_accuracy);
    }
}

fn print_metadata(meta: &protobufs::DeviceMetadata) {
    let hw_model = HardwareModel::try_from(meta.hw_model)
        .map(|m| m.as_str_name().to_string())
        .unwrap_or_else(|_| format!("Unknown({})", meta.hw_model));

    let role = protobufs::config::device_config::Role::try_from(meta.role)
        .map(|r| format!("{:?}", r))
        .unwrap_or_else(|_| meta.role.to_string());

    println!(
        "  {:<28} {}",
        "firmware_version:".dimmed(),
        meta.firmware_version
    );
    println!(
        "  {:<28} {}",
        "device_state_version:".dimmed(),
        meta.device_state_version
    );
    println!("  {:<28} {}", "hw_model:".dimmed(), hw_model);
    println!("  {:<28} {}", "role:".dimmed(), role);
    println!("  {:<28} {}", "can_shutdown:".dimmed(), meta.can_shutdown);
    println!("  {:<28} {}", "has_wifi:".dimmed(), meta.has_wifi);
    println!("  {:<28} {}", "has_bluetooth:".dimmed(), meta.has_bluetooth);
    println!("  {:<28} {}", "has_ethernet:".dimmed(), meta.has_ethernet);
    println!(
        "  {:<28} {}",
        "has_remote_hardware:".dimmed(),
        meta.has_remote_hardware
    );
    println!("  {:<28} {}", "has_pkc:".dimmed(), meta.has_pkc);
}

fn telemetry_to_json(telem: &Telemetry, source: &str, rtt_s: f64) -> serde_json::Value {
    let data = match &telem.variant {
        Some(telemetry::Variant::DeviceMetrics(m)) => json!({
            "type": "device",
            "battery_level": m.battery_level,
            "voltage": m.voltage,
            "channel_utilization": m.channel_utilization,
            "air_util_tx": m.air_util_tx,
            "uptime_seconds": m.uptime_seconds,
        }),
        Some(telemetry::Variant::EnvironmentMetrics(m)) => json!({
            "type": "environment",
            "temperature": m.temperature,
            "relative_humidity": m.relative_humidity,
            "barometric_pressure": m.barometric_pressure,
        }),
        Some(telemetry::Variant::PowerMetrics(m)) => json!({
            "type": "power",
            "ch1_voltage": m.ch1_voltage,
            "ch1_current": m.ch1_current,
            "ch2_voltage": m.ch2_voltage,
            "ch2_current": m.ch2_current,
            "ch3_voltage": m.ch3_voltage,
            "ch3_current": m.ch3_current,
        }),
        Some(telemetry::Variant::AirQualityMetrics(m)) => json!({
            "type": "air_quality",
            "pm10_standard": m.pm10_standard,
            "pm25_standard": m.pm25_standard,
            "pm100_standard": m.pm100_standard,
            "co2": m.co2,
        }),
        Some(telemetry::Variant::LocalStats(m)) => json!({
            "type": "local_stats",
            "uptime_seconds": m.uptime_seconds,
            "channel_utilization": m.channel_utilization,
            "air_util_tx": m.air_util_tx,
            "packets_tx": m.num_packets_tx,
            "packets_rx": m.num_packets_rx,
            "packets_rx_bad": m.num_packets_rx_bad,
            "online_nodes": m.num_online_nodes,
            "total_nodes": m.num_total_nodes,
        }),
        Some(telemetry::Variant::HealthMetrics(m)) => json!({
            "type": "health",
            "heart_bpm": m.heart_bpm,
            "sp_o2": m.sp_o2,
        }),
        Some(telemetry::Variant::HostMetrics(m)) => json!({
            "type": "host",
            "uptime_seconds": m.uptime_seconds,
            "freemem_bytes": m.freemem_bytes,
            "diskfree1_bytes": m.diskfree1_bytes,
        }),
        None => json!({"type": "empty"}),
    };
    json!({
        "source": source,
        "rtt_s": rtt_s,
        "telemetry": data,
    })
}

fn format_uptime(seconds: u32) -> String {
    let d = seconds / 86400;
    let h = (seconds % 86400) / 3600;
    let m = (seconds % 3600) / 60;
    if d > 0 {
        format!("{}d {}h {}m", d, h, m)
    } else if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}
