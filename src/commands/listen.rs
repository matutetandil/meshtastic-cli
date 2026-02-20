use async_trait::async_trait;
use chrono::{DateTime, Local, Utc};
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::telemetry::Variant as TelemetryVariant;
use meshtastic::protobufs::{MeshPacket, PortNum, Position, Routing, Telemetry, User};
use meshtastic::Message;

use super::{Command, CommandContext};
use crate::node_db::NodeDb;

const BROADCAST_ADDR: u32 = 0xFFFFFFFF;

pub struct ListenCommand;

#[async_trait]
impl Command for ListenCommand {
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        println!(
            "{} Listening for packets... Press {} to stop.\n",
            "→".cyan(),
            "Ctrl+C".bold()
        );

        while let Some(packet) = ctx.packet_receiver.recv().await {
            let Some(PayloadVariant::Packet(mesh_packet)) = packet.payload_variant else {
                continue;
            };

            print_packet(&mesh_packet, &ctx.node_db);
        }

        println!("\nDisconnected from device.");
        Ok(())
    }
}

fn print_packet(packet: &MeshPacket, node_db: &NodeDb) {
    let timestamp = format_timestamp(packet.rx_time);
    let from = format_node(packet.from, node_db);
    let to = format_destination(packet.to, node_db);
    let channel = packet.channel;

    let Some(MeshPayload::Decoded(ref data)) = packet.payload_variant else {
        println!(
            "{} {} {} {} | {}",
            timestamp.dimmed(),
            from,
            "→".dimmed(),
            to,
            "Encrypted packet".dimmed()
        );
        return;
    };

    let port = PortNum::try_from(data.portnum).unwrap_or(PortNum::UnknownApp);
    let content = format_payload(&port, &data.payload);

    let port_label = format_port_label(&port);
    let channel_info = if channel > 0 {
        format!(" ch:{}", channel)
    } else {
        String::new()
    };

    println!(
        "{} {} {} {}{} | {} {}",
        timestamp.dimmed(),
        from,
        "→".dimmed(),
        to,
        channel_info.dimmed(),
        port_label,
        content
    );
}

fn format_timestamp(rx_time: u32) -> String {
    if rx_time == 0 {
        return "          ".to_string();
    }
    DateTime::<Utc>::from_timestamp(rx_time as i64, 0)
        .map(|dt| {
            let local: DateTime<Local> = dt.into();
            local.format("[%H:%M:%S]").to_string()
        })
        .unwrap_or_else(|| "          ".to_string())
}

fn format_node(node_num: u32, node_db: &NodeDb) -> String {
    let id = format!("!{:08x}", node_num);
    match node_db.node_name(node_num) {
        Some(name) => format!("{} ({})", id, name),
        None => id,
    }
}

fn format_destination(node_num: u32, node_db: &NodeDb) -> String {
    if node_num == BROADCAST_ADDR {
        "broadcast".to_string()
    } else {
        format_node(node_num, node_db)
    }
}

fn format_port_label(port: &PortNum) -> String {
    let label = match port {
        PortNum::TextMessageApp => "Text:",
        PortNum::PositionApp => "Position:",
        PortNum::NodeinfoApp => "NodeInfo:",
        PortNum::RoutingApp => "Routing:",
        PortNum::TelemetryApp => "Telemetry:",
        PortNum::WaypointApp => "Waypoint:",
        PortNum::TracerouteApp => "Traceroute:",
        PortNum::NeighborinfoApp => "NeighborInfo:",
        PortNum::DetectionSensorApp => "Detection:",
        PortNum::RangeTestApp => "RangeTest:",
        PortNum::StoreForwardApp => "StoreForward:",
        PortNum::AdminApp => "Admin:",
        _ => "Unknown:",
    };
    label.yellow().to_string()
}

fn format_payload(port: &PortNum, payload: &[u8]) -> String {
    match port {
        PortNum::TextMessageApp => format_text(payload),
        PortNum::PositionApp => format_position(payload),
        PortNum::NodeinfoApp => format_nodeinfo(payload),
        PortNum::RoutingApp => format_routing(payload),
        PortNum::TelemetryApp => format_telemetry(payload),
        _ => format!("{} bytes", payload.len()),
    }
}

fn format_text(payload: &[u8]) -> String {
    String::from_utf8(payload.to_vec())
        .unwrap_or_else(|_| format!("<invalid UTF-8: {} bytes>", payload.len()))
}

fn format_position(payload: &[u8]) -> String {
    let Ok(pos) = Position::decode(payload) else {
        return format!("<decode error: {} bytes>", payload.len());
    };

    let lat = pos.latitude_i.unwrap_or(0) as f64 * 1e-7;
    let lon = pos.longitude_i.unwrap_or(0) as f64 * 1e-7;
    let alt = pos.altitude.unwrap_or(0);
    let sats = pos.sats_in_view;

    let mut parts = vec![format!("{:.5}, {:.5}", lat, lon)];
    if alt != 0 {
        parts.push(format!("{}m", alt));
    }
    if sats > 0 {
        parts.push(format!("{} sats", sats));
    }
    parts.join(", ")
}

fn format_nodeinfo(payload: &[u8]) -> String {
    let Ok(user) = User::decode(payload) else {
        return format!("<decode error: {} bytes>", payload.len());
    };
    format!("{} ({})", user.long_name, user.short_name)
}

fn format_routing(payload: &[u8]) -> String {
    let Ok(routing) = Routing::decode(payload) else {
        return format!("<decode error: {} bytes>", payload.len());
    };

    match routing.variant {
        Some(meshtastic::protobufs::routing::Variant::ErrorReason(code)) => {
            let reason = match code {
                0 => "ACK",
                1 => "NO_ROUTE",
                2 => "GOT_NAK",
                3 => "TIMEOUT",
                4 => "NO_INTERFACE",
                5 => "MAX_RETRANSMIT",
                6 => "NO_CHANNEL",
                7 => "TOO_LARGE",
                8 => "NO_RESPONSE",
                9 => "DUTY_CYCLE_LIMIT",
                _ => "UNKNOWN",
            };
            reason.to_string()
        }
        Some(meshtastic::protobufs::routing::Variant::RouteRequest(_)) => {
            "Route request".to_string()
        }
        Some(meshtastic::protobufs::routing::Variant::RouteReply(_)) => "Route reply".to_string(),
        None => "Unknown routing".to_string(),
    }
}

fn format_telemetry(payload: &[u8]) -> String {
    let Ok(telemetry) = Telemetry::decode(payload) else {
        return format!("<decode error: {} bytes>", payload.len());
    };

    match telemetry.variant {
        Some(TelemetryVariant::DeviceMetrics(m)) => {
            let mut parts = Vec::new();
            if let Some(bat) = m.battery_level {
                parts.push(format!("battery {}%", bat));
            }
            if let Some(v) = m.voltage {
                parts.push(format!("{:.2}V", v));
            }
            if let Some(cu) = m.channel_utilization {
                parts.push(format!("ch_util {:.1}%", cu));
            }
            if let Some(at) = m.air_util_tx {
                parts.push(format!("air_tx {:.1}%", at));
            }
            if let Some(up) = m.uptime_seconds {
                parts.push(format!("uptime {}s", up));
            }
            if parts.is_empty() {
                "device metrics (empty)".to_string()
            } else {
                parts.join(", ")
            }
        }
        Some(TelemetryVariant::EnvironmentMetrics(m)) => {
            let mut parts = Vec::new();
            if let Some(t) = m.temperature {
                parts.push(format!("{:.1}C", t));
            }
            if let Some(h) = m.relative_humidity {
                parts.push(format!("{:.1}% humidity", h));
            }
            if let Some(p) = m.barometric_pressure {
                parts.push(format!("{:.1} hPa", p));
            }
            if parts.is_empty() {
                "env metrics (empty)".to_string()
            } else {
                parts.join(", ")
            }
        }
        Some(TelemetryVariant::PowerMetrics(m)) => {
            format!(
                "power: ch1 {:.2}V/{:.1}mA, ch2 {:.2}V/{:.1}mA",
                m.ch1_voltage.unwrap_or(0.0),
                m.ch1_current.unwrap_or(0.0),
                m.ch2_voltage.unwrap_or(0.0),
                m.ch2_current.unwrap_or(0.0)
            )
        }
        Some(TelemetryVariant::AirQualityMetrics(_)) => "air quality metrics".to_string(),
        Some(TelemetryVariant::LocalStats(_)) => "local stats".to_string(),
        Some(TelemetryVariant::HealthMetrics(_)) => "health metrics".to_string(),
        Some(TelemetryVariant::HostMetrics(_)) => "host metrics".to_string(),
        None => "telemetry (no data)".to_string(),
    }
}
