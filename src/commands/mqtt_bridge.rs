use std::time::Duration;

use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::from_radio::PayloadVariant;
use meshtastic::protobufs::mesh_packet::PayloadVariant as MeshPayload;
use meshtastic::protobufs::telemetry::Variant as TelemetryVariant;
use meshtastic::protobufs::{PortNum, Position, Telemetry};
use meshtastic::Message;
use rumqttc::{AsyncClient, MqttOptions, QoS};

use super::{Command, CommandContext};

pub struct MqttBridgeCommand {
    pub broker: String,
    pub port: u16,
    pub topic_prefix: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub json: bool,
}

#[async_trait]
impl Command for MqttBridgeCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let client_id = format!("meshtastic-cli-{}", rand_u16());

        let mut mqtt_opts = MqttOptions::new(&client_id, &self.broker, self.port);
        mqtt_opts.set_keep_alive(Duration::from_secs(30));

        if let (Some(user), Some(pass)) = (&self.username, &self.password) {
            mqtt_opts.set_credentials(user, pass);
        }

        let (client, mut eventloop) = AsyncClient::new(mqtt_opts, 64);

        // Subscribe to the send topic for MQTT→Mesh messages
        let send_topic = format!("{}/send", self.topic_prefix);
        client.subscribe(&send_topic, QoS::AtLeastOnce).await?;

        if !self.json {
            println!(
                "{} MQTT bridge started: {}:{}",
                "->".cyan(),
                self.broker.bold(),
                self.port
            );
            println!("  {:<16} {}", "topic_prefix:".dimmed(), self.topic_prefix);
            println!("  {:<16} {}", "subscribe:".dimmed(), send_topic);
            println!(
                "\n{} Bridging mesh ↔ MQTT... Press {} to stop.\n",
                "->".cyan(),
                "Ctrl+C".bold()
            );
        }

        let json = self.json;
        let prefix = self.topic_prefix.clone();
        let client_pub = client.clone();

        // Spawn MQTT event loop handler (MQTT→Mesh)
        let api_clone = &ctx.api;
        let _ = api_clone; // We'll handle MQTT→Mesh in the select loop below

        loop {
            tokio::select! {
                // Mesh→MQTT: forward mesh packets to MQTT topics
                packet = ctx.packet_receiver.recv() => {
                    let Some(from_radio) = packet else {
                        if !json {
                            println!("\n{} Disconnected from mesh device.", "x".red());
                        }
                        break;
                    };

                    let Some(PayloadVariant::Packet(mesh_pkt)) = from_radio.payload_variant else {
                        continue;
                    };
                    let Some(MeshPayload::Decoded(ref data)) = mesh_pkt.payload_variant else {
                        continue;
                    };

                    let from = format!("!{:08x}", mesh_pkt.from);
                    let port = PortNum::try_from(data.portnum).unwrap_or(PortNum::UnknownApp);

                    match port {
                        PortNum::TextMessageApp => {
                            let text = String::from_utf8_lossy(&data.payload);
                            let topic = format!("{}/messages", prefix);
                            let payload = serde_json::json!({
                                "from": from,
                                "to": format!("!{:08x}", mesh_pkt.to),
                                "text": text,
                                "channel": mesh_pkt.channel,
                                "rx_time": mesh_pkt.rx_time,
                            });
                            let msg = serde_json::to_string(&payload).unwrap_or_default();
                            let _ = client_pub.publish(&topic, QoS::AtLeastOnce, false, msg).await;

                            if !json {
                                println!(
                                    "{} → MQTT {}: {} from {}",
                                    ">>".green(),
                                    "messages".dimmed(),
                                    text,
                                    from
                                );
                            }
                        }
                        PortNum::TelemetryApp => {
                            if let Ok(telemetry) = Telemetry::decode(data.payload.as_slice()) {
                                let topic = format!("{}/telemetry/{}", prefix, from);
                                let payload = telemetry_to_json(&telemetry);
                                let msg = serde_json::to_string(&payload).unwrap_or_default();
                                let _ = client_pub.publish(&topic, QoS::AtLeastOnce, false, msg).await;

                                if !json {
                                    println!(
                                        "{} → MQTT {}: telemetry from {}",
                                        ">>".green(),
                                        "telemetry".dimmed(),
                                        from
                                    );
                                }
                            }
                        }
                        PortNum::PositionApp => {
                            if let Ok(pos) = Position::decode(data.payload.as_slice()) {
                                let topic = format!("{}/position/{}", prefix, from);
                                let lat = pos.latitude_i.unwrap_or(0) as f64 / 1e7;
                                let lon = pos.longitude_i.unwrap_or(0) as f64 / 1e7;
                                let payload = serde_json::json!({
                                    "from": from,
                                    "latitude": lat,
                                    "longitude": lon,
                                    "altitude": pos.altitude.unwrap_or(0),
                                    "sats_in_view": pos.sats_in_view,
                                });
                                let msg = serde_json::to_string(&payload).unwrap_or_default();
                                let _ = client_pub.publish(&topic, QoS::AtLeastOnce, false, msg).await;

                                if !json {
                                    println!(
                                        "{} → MQTT {}: position from {}",
                                        ">>".green(),
                                        "position".dimmed(),
                                        from
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // MQTT event loop: handle incoming MQTT messages
                event = eventloop.poll() => {
                    match event {
                        Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(msg))) => {
                            if msg.topic == send_topic {
                                if let Ok(text) = String::from_utf8(msg.payload.to_vec()) {
                                    // Try parsing as JSON with optional dest/channel
                                    let send_msg: MqttSendMessage = serde_json::from_str(&text)
                                        .unwrap_or(MqttSendMessage {
                                            text: text.clone(),
                                            dest: None,
                                            channel: None,
                                        });

                                    let dest = send_msg.dest.as_ref().and_then(|d| {
                                        let stripped = d.strip_prefix('!').unwrap_or(d);
                                        u32::from_str_radix(stripped, 16).ok()
                                    });

                                    let channel = send_msg.channel.unwrap_or(0);
                                    let mesh_channel = meshtastic::types::MeshChannel::new(channel)
                                        .unwrap_or_else(|_| meshtastic::types::MeshChannel::new(0).unwrap());

                                    let packet_dest = match dest {
                                        Some(id) => meshtastic::packet::PacketDestination::Node(
                                            meshtastic::types::NodeId::new(id),
                                        ),
                                        None => meshtastic::packet::PacketDestination::Broadcast,
                                    };

                                    match ctx.api.send_text(
                                        &mut ctx.router,
                                        send_msg.text.clone(),
                                        packet_dest,
                                        true,
                                        mesh_channel,
                                    ).await {
                                        Ok(_) => {
                                            if !json {
                                                println!(
                                                    "{} ← MQTT: sent '{}'",
                                                    "<<".blue(),
                                                    send_msg.text
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            if !json {
                                                println!(
                                                    "{} MQTT→Mesh send failed: {}",
                                                    "x".red(),
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            if !json {
                                println!("{} MQTT error: {}", "x".red(), e);
                            }
                            // Reconnect is handled automatically by rumqttc
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct MqttSendMessage {
    text: String,
    #[serde(default)]
    dest: Option<String>,
    #[serde(default)]
    channel: Option<u32>,
}

fn telemetry_to_json(telemetry: &Telemetry) -> serde_json::Value {
    match &telemetry.variant {
        Some(TelemetryVariant::DeviceMetrics(m)) => serde_json::json!({
            "type": "device",
            "battery_level": m.battery_level,
            "voltage": m.voltage,
            "channel_utilization": m.channel_utilization,
            "air_util_tx": m.air_util_tx,
            "uptime_seconds": m.uptime_seconds,
        }),
        Some(TelemetryVariant::EnvironmentMetrics(m)) => serde_json::json!({
            "type": "environment",
            "temperature": m.temperature,
            "relative_humidity": m.relative_humidity,
            "barometric_pressure": m.barometric_pressure,
        }),
        Some(TelemetryVariant::PowerMetrics(m)) => serde_json::json!({
            "type": "power",
            "ch1_voltage": m.ch1_voltage,
            "ch1_current": m.ch1_current,
            "ch2_voltage": m.ch2_voltage,
            "ch2_current": m.ch2_current,
        }),
        _ => serde_json::json!({ "type": "unknown" }),
    }
}

fn rand_u16() -> u16 {
    use std::time::SystemTime;
    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    (seed & 0xFFFF) as u16
}
