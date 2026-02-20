use std::time::{Duration, Instant};

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::{self, telemetry, Data, MeshPacket, PortNum, Position, Telemetry};
use meshtastic::utils::generate_rand_id;
use meshtastic::Message;

use super::{resolve_destination, Command, CommandContext, DestinationSpec};

// ── RequestTelemetryCommand ───────────────────────────────────────

pub struct RequestTelemetryCommand {
    pub destination: DestinationSpec,
    pub timeout_secs: u64,
}

#[async_trait]
impl Command for RequestTelemetryCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let (packet_dest, dest_label) = resolve_destination(&self.destination, &ctx.node_db)?;

        let target_id = match packet_dest {
            meshtastic::packet::PacketDestination::Node(node_id) => node_id.id(),
            _ => bail!("Request requires a specific node destination"),
        };

        let packet_id = generate_rand_id();
        let my_node = ctx.node_db.my_node_num();

        let telemetry = Telemetry {
            time: 0,
            variant: None,
        };

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
                        println!(
                            "{} Telemetry from {} (in {:.1}s):",
                            "ok".green(),
                            dest_label,
                            elapsed
                        );
                        print_telemetry(&telem);
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
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
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
                        println!(
                            "{} Position from {} (in {:.1}s):",
                            "ok".green(),
                            dest_label,
                            elapsed
                        );
                        print_position(&pos);
                        return Ok(());
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
        }
        None => {
            println!("  {}", "(empty telemetry response)".dimmed());
        }
        _ => {
            println!("  (other telemetry variant)");
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
