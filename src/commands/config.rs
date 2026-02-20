use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs;
use meshtastic::Message;

use crate::cli::ConfigSection;

use super::{Command, CommandContext};

// ── ConfigGetCommand ───────────────────────────────────────────────

pub struct ConfigGetCommand {
    pub section: Option<ConfigSection>,
}

#[async_trait]
impl Command for ConfigGetCommand {
    async fn execute(self: Box<Self>, ctx: CommandContext) -> anyhow::Result<()> {
        let config = ctx.node_db.local_config();
        let module = ctx.node_db.local_module_config();

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
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
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
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
        let short_name = self
            .short_name
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
        super::device::send_admin_message(
            &mut ctx,
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
    async fn execute(self: Box<Self>, mut ctx: CommandContext) -> anyhow::Result<()> {
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

// ── Print helpers ──────────────────────────────────────────────────

fn section_header(name: &str) {
    println!("{}", name.bold().underline());
}

fn field(label: &str, value: &str) {
    println!("  {:<40} {}", format!("{}:", label).dimmed(), value);
}

fn field_enum<T: std::fmt::Debug>(label: &str, value: i32, f: impl FnOnce(i32) -> Option<T>) {
    let display = f(value)
        .map(|v| format!("{:?}", v))
        .unwrap_or_else(|| value.to_string());
    field(label, &display);
}

fn not_received(section_name: &str) {
    section_header(section_name);
    println!("  {}", "(not received from device)".dimmed());
    println!();
}

// ── Config section printers ────────────────────────────────────────

fn print_device(cfg: Option<&protobufs::config::DeviceConfig>) {
    let Some(c) = cfg else {
        return not_received("Device");
    };
    section_header("Device");
    field_enum("role", c.role, |v| {
        protobufs::config::device_config::Role::try_from(v).ok()
    });
    field(
        "node_info_broadcast_secs",
        &c.node_info_broadcast_secs.to_string(),
    );
    field(
        "double_tap_as_button_press",
        &c.double_tap_as_button_press.to_string(),
    );
    field("disable_triple_click", &c.disable_triple_click.to_string());
    field(
        "led_heartbeat_disabled",
        &c.led_heartbeat_disabled.to_string(),
    );
    field("button_gpio", &c.button_gpio.to_string());
    field("buzzer_gpio", &c.buzzer_gpio.to_string());
    field_enum("rebroadcast_mode", c.rebroadcast_mode, |v| {
        protobufs::config::device_config::RebroadcastMode::try_from(v).ok()
    });
    field_enum("buzzer_mode", c.buzzer_mode, |v| {
        protobufs::config::device_config::BuzzerMode::try_from(v).ok()
    });
    if !c.tzdef.is_empty() {
        field("tzdef", &c.tzdef);
    }
    println!();
}

fn print_position(cfg: Option<&protobufs::config::PositionConfig>) {
    let Some(c) = cfg else {
        return not_received("Position");
    };
    section_header("Position");
    field(
        "position_broadcast_secs",
        &c.position_broadcast_secs.to_string(),
    );
    field(
        "position_broadcast_smart_enabled",
        &c.position_broadcast_smart_enabled.to_string(),
    );
    field("fixed_position", &c.fixed_position.to_string());
    field("gps_update_interval", &c.gps_update_interval.to_string());
    field("position_flags", &c.position_flags.to_string());
    field("rx_gpio", &c.rx_gpio.to_string());
    field("tx_gpio", &c.tx_gpio.to_string());
    field(
        "broadcast_smart_minimum_distance",
        &c.broadcast_smart_minimum_distance.to_string(),
    );
    field(
        "broadcast_smart_minimum_interval_secs",
        &c.broadcast_smart_minimum_interval_secs.to_string(),
    );
    field("gps_en_gpio", &c.gps_en_gpio.to_string());
    field_enum("gps_mode", c.gps_mode, |v| {
        protobufs::config::position_config::GpsMode::try_from(v).ok()
    });
    println!();
}

fn print_power(cfg: Option<&protobufs::config::PowerConfig>) {
    let Some(c) = cfg else {
        return not_received("Power");
    };
    section_header("Power");
    field("is_power_saving", &c.is_power_saving.to_string());
    field(
        "on_battery_shutdown_after_secs",
        &c.on_battery_shutdown_after_secs.to_string(),
    );
    field(
        "adc_multiplier_override",
        &c.adc_multiplier_override.to_string(),
    );
    field("wait_bluetooth_secs", &c.wait_bluetooth_secs.to_string());
    field("sds_secs", &c.sds_secs.to_string());
    field("ls_secs", &c.ls_secs.to_string());
    field("min_wake_secs", &c.min_wake_secs.to_string());
    field(
        "device_battery_ina_address",
        &c.device_battery_ina_address.to_string(),
    );
    field("powermon_enables", &c.powermon_enables.to_string());
    println!();
}

fn print_network(cfg: Option<&protobufs::config::NetworkConfig>) {
    let Some(c) = cfg else {
        return not_received("Network");
    };
    section_header("Network");
    field("wifi_enabled", &c.wifi_enabled.to_string());
    field("wifi_ssid", &c.wifi_ssid);
    field(
        "wifi_psk",
        if c.wifi_psk.is_empty() {
            "(empty)"
        } else {
            &c.wifi_psk
        },
    );
    field("ntp_server", &c.ntp_server);
    field("eth_enabled", &c.eth_enabled.to_string());
    field_enum("address_mode", c.address_mode, |v| {
        protobufs::config::network_config::AddressMode::try_from(v).ok()
    });
    field("rsyslog_server", &c.rsyslog_server);
    field("ipv6_enabled", &c.ipv6_enabled.to_string());
    println!();
}

fn print_display(cfg: Option<&protobufs::config::DisplayConfig>) {
    let Some(c) = cfg else {
        return not_received("Display");
    };
    section_header("Display");
    field("screen_on_secs", &c.screen_on_secs.to_string());
    field(
        "auto_screen_carousel_secs",
        &c.auto_screen_carousel_secs.to_string(),
    );
    field("flip_screen", &c.flip_screen.to_string());
    field_enum("units", c.units, |v| {
        protobufs::config::display_config::DisplayUnits::try_from(v).ok()
    });
    field_enum("oled", c.oled, |v| {
        protobufs::config::display_config::OledType::try_from(v).ok()
    });
    field_enum("displaymode", c.displaymode, |v| {
        protobufs::config::display_config::DisplayMode::try_from(v).ok()
    });
    field("heading_bold", &c.heading_bold.to_string());
    field(
        "wake_on_tap_or_motion",
        &c.wake_on_tap_or_motion.to_string(),
    );
    field_enum("compass_orientation", c.compass_orientation, |v| {
        protobufs::config::display_config::CompassOrientation::try_from(v).ok()
    });
    field("use_12h_clock", &c.use_12h_clock.to_string());
    println!();
}

fn print_lora(cfg: Option<&protobufs::config::LoRaConfig>) {
    let Some(c) = cfg else {
        return not_received("LoRa");
    };
    section_header("LoRa");
    field_enum("region", c.region, |v| {
        protobufs::config::lo_ra_config::RegionCode::try_from(v).ok()
    });
    field_enum("modem_preset", c.modem_preset, |v| {
        protobufs::config::lo_ra_config::ModemPreset::try_from(v).ok()
    });
    field("use_preset", &c.use_preset.to_string());
    field("bandwidth", &c.bandwidth.to_string());
    field("spread_factor", &c.spread_factor.to_string());
    field("coding_rate", &c.coding_rate.to_string());
    field("frequency_offset", &c.frequency_offset.to_string());
    field("hop_limit", &c.hop_limit.to_string());
    field("tx_enabled", &c.tx_enabled.to_string());
    field("tx_power", &c.tx_power.to_string());
    field("channel_num", &c.channel_num.to_string());
    field("override_duty_cycle", &c.override_duty_cycle.to_string());
    field(
        "sx126x_rx_boosted_gain",
        &c.sx126x_rx_boosted_gain.to_string(),
    );
    field("override_frequency", &c.override_frequency.to_string());
    field("pa_fan_disabled", &c.pa_fan_disabled.to_string());
    field("ignore_mqtt", &c.ignore_mqtt.to_string());
    field("config_ok_to_mqtt", &c.config_ok_to_mqtt.to_string());
    println!();
}

fn print_bluetooth(cfg: Option<&protobufs::config::BluetoothConfig>) {
    let Some(c) = cfg else {
        return not_received("Bluetooth");
    };
    section_header("Bluetooth");
    field("enabled", &c.enabled.to_string());
    field_enum("mode", c.mode, |v| {
        protobufs::config::bluetooth_config::PairingMode::try_from(v).ok()
    });
    field("fixed_pin", &c.fixed_pin.to_string());
    println!();
}

fn print_security(cfg: Option<&protobufs::config::SecurityConfig>) {
    let Some(c) = cfg else {
        return not_received("Security");
    };
    section_header("Security");
    field("is_managed", &c.is_managed.to_string());
    field("serial_enabled", &c.serial_enabled.to_string());
    field(
        "debug_log_api_enabled",
        &c.debug_log_api_enabled.to_string(),
    );
    field(
        "admin_channel_enabled",
        &c.admin_channel_enabled.to_string(),
    );
    field(
        "public_key",
        &if c.public_key.is_empty() {
            "(empty)".to_string()
        } else {
            format!("{} bytes", c.public_key.len())
        },
    );
    field(
        "private_key",
        &if c.private_key.is_empty() {
            "(empty)".to_string()
        } else {
            format!("{} bytes", c.private_key.len())
        },
    );
    field("admin_key", &format!("{} entries", c.admin_key.len()));
    println!();
}

// ── Module config printers ─────────────────────────────────────────

fn print_mqtt(cfg: Option<&protobufs::module_config::MqttConfig>) {
    let Some(c) = cfg else {
        return not_received("MQTT");
    };
    section_header("MQTT");
    field("enabled", &c.enabled.to_string());
    field("address", &c.address);
    field("username", &c.username);
    field(
        "password",
        if c.password.is_empty() {
            "(empty)"
        } else {
            "***"
        },
    );
    field("encryption_enabled", &c.encryption_enabled.to_string());
    field("json_enabled", &c.json_enabled.to_string());
    field("tls_enabled", &c.tls_enabled.to_string());
    field("root", &c.root);
    field(
        "proxy_to_client_enabled",
        &c.proxy_to_client_enabled.to_string(),
    );
    field(
        "map_reporting_enabled",
        &c.map_reporting_enabled.to_string(),
    );
    println!();
}

fn print_serial(cfg: Option<&protobufs::module_config::SerialConfig>) {
    let Some(c) = cfg else {
        return not_received("Serial");
    };
    section_header("Serial");
    field("enabled", &c.enabled.to_string());
    field("echo", &c.echo.to_string());
    field("rxd", &c.rxd.to_string());
    field("txd", &c.txd.to_string());
    field_enum("baud", c.baud, |v| {
        protobufs::module_config::serial_config::SerialBaud::try_from(v).ok()
    });
    field("timeout", &c.timeout.to_string());
    field_enum("mode", c.mode, |v| {
        protobufs::module_config::serial_config::SerialMode::try_from(v).ok()
    });
    field(
        "override_console_serial_port",
        &c.override_console_serial_port.to_string(),
    );
    println!();
}

fn print_external_notification(cfg: Option<&protobufs::module_config::ExternalNotificationConfig>) {
    let Some(c) = cfg else {
        return not_received("External Notification");
    };
    section_header("External Notification");
    field("enabled", &c.enabled.to_string());
    field("output_ms", &c.output_ms.to_string());
    field("output", &c.output.to_string());
    field("output_vibra", &c.output_vibra.to_string());
    field("output_buzzer", &c.output_buzzer.to_string());
    field("active", &c.active.to_string());
    field("alert_message", &c.alert_message.to_string());
    field("alert_message_vibra", &c.alert_message_vibra.to_string());
    field("alert_message_buzzer", &c.alert_message_buzzer.to_string());
    field("alert_bell", &c.alert_bell.to_string());
    field("alert_bell_vibra", &c.alert_bell_vibra.to_string());
    field("alert_bell_buzzer", &c.alert_bell_buzzer.to_string());
    field("use_pwm", &c.use_pwm.to_string());
    field("nag_timeout", &c.nag_timeout.to_string());
    field("use_i2s_as_buzzer", &c.use_i2s_as_buzzer.to_string());
    println!();
}

fn print_store_forward(cfg: Option<&protobufs::module_config::StoreForwardConfig>) {
    let Some(c) = cfg else {
        return not_received("Store & Forward");
    };
    section_header("Store & Forward");
    field("enabled", &c.enabled.to_string());
    field("heartbeat", &c.heartbeat.to_string());
    field("records", &c.records.to_string());
    field("history_return_max", &c.history_return_max.to_string());
    field(
        "history_return_window",
        &c.history_return_window.to_string(),
    );
    field("is_server", &c.is_server.to_string());
    println!();
}

fn print_range_test(cfg: Option<&protobufs::module_config::RangeTestConfig>) {
    let Some(c) = cfg else {
        return not_received("Range Test");
    };
    section_header("Range Test");
    field("enabled", &c.enabled.to_string());
    field("sender", &c.sender.to_string());
    field("save", &c.save.to_string());
    println!();
}

fn print_telemetry(cfg: Option<&protobufs::module_config::TelemetryConfig>) {
    let Some(c) = cfg else {
        return not_received("Telemetry");
    };
    section_header("Telemetry");
    field(
        "device_update_interval",
        &c.device_update_interval.to_string(),
    );
    field(
        "environment_update_interval",
        &c.environment_update_interval.to_string(),
    );
    field(
        "environment_measurement_enabled",
        &c.environment_measurement_enabled.to_string(),
    );
    field(
        "environment_screen_enabled",
        &c.environment_screen_enabled.to_string(),
    );
    field(
        "environment_display_fahrenheit",
        &c.environment_display_fahrenheit.to_string(),
    );
    field("air_quality_enabled", &c.air_quality_enabled.to_string());
    field("air_quality_interval", &c.air_quality_interval.to_string());
    field(
        "power_measurement_enabled",
        &c.power_measurement_enabled.to_string(),
    );
    field(
        "power_update_interval",
        &c.power_update_interval.to_string(),
    );
    field("power_screen_enabled", &c.power_screen_enabled.to_string());
    field(
        "health_measurement_enabled",
        &c.health_measurement_enabled.to_string(),
    );
    field(
        "health_update_interval",
        &c.health_update_interval.to_string(),
    );
    field(
        "health_screen_enabled",
        &c.health_screen_enabled.to_string(),
    );
    println!();
}

fn print_canned_message(cfg: Option<&protobufs::module_config::CannedMessageConfig>) {
    let Some(c) = cfg else {
        return not_received("Canned Message");
    };
    section_header("Canned Message");
    field("rotary1_enabled", &c.rotary1_enabled.to_string());
    field("inputbroker_pin_a", &c.inputbroker_pin_a.to_string());
    field("inputbroker_pin_b", &c.inputbroker_pin_b.to_string());
    field(
        "inputbroker_pin_press",
        &c.inputbroker_pin_press.to_string(),
    );
    field_enum("inputbroker_event_cw", c.inputbroker_event_cw, |v| {
        protobufs::module_config::canned_message_config::InputEventChar::try_from(v).ok()
    });
    field_enum("inputbroker_event_ccw", c.inputbroker_event_ccw, |v| {
        protobufs::module_config::canned_message_config::InputEventChar::try_from(v).ok()
    });
    field_enum("inputbroker_event_press", c.inputbroker_event_press, |v| {
        protobufs::module_config::canned_message_config::InputEventChar::try_from(v).ok()
    });
    field("updown1_enabled", &c.updown1_enabled.to_string());
    field("send_bell", &c.send_bell.to_string());
    println!();
}

fn print_audio(cfg: Option<&protobufs::module_config::AudioConfig>) {
    let Some(c) = cfg else {
        return not_received("Audio");
    };
    section_header("Audio");
    field("codec2_enabled", &c.codec2_enabled.to_string());
    field("ptt_pin", &c.ptt_pin.to_string());
    field_enum("bitrate", c.bitrate, |v| {
        protobufs::module_config::audio_config::AudioBaud::try_from(v).ok()
    });
    field("i2s_ws", &c.i2s_ws.to_string());
    field("i2s_sd", &c.i2s_sd.to_string());
    field("i2s_din", &c.i2s_din.to_string());
    field("i2s_sck", &c.i2s_sck.to_string());
    println!();
}

fn print_remote_hardware(cfg: Option<&protobufs::module_config::RemoteHardwareConfig>) {
    let Some(c) = cfg else {
        return not_received("Remote Hardware");
    };
    section_header("Remote Hardware");
    field("enabled", &c.enabled.to_string());
    field(
        "allow_undefined_pin_access",
        &c.allow_undefined_pin_access.to_string(),
    );
    field(
        "available_pins",
        &format!("{} entries", c.available_pins.len()),
    );
    println!();
}

fn print_neighbor_info(cfg: Option<&protobufs::module_config::NeighborInfoConfig>) {
    let Some(c) = cfg else {
        return not_received("Neighbor Info");
    };
    section_header("Neighbor Info");
    field("enabled", &c.enabled.to_string());
    field("update_interval", &c.update_interval.to_string());
    field("transmit_over_lora", &c.transmit_over_lora.to_string());
    println!();
}

fn print_ambient_lighting(cfg: Option<&protobufs::module_config::AmbientLightingConfig>) {
    let Some(c) = cfg else {
        return not_received("Ambient Lighting");
    };
    section_header("Ambient Lighting");
    field("led_state", &c.led_state.to_string());
    field("current", &c.current.to_string());
    field("red", &c.red.to_string());
    field("green", &c.green.to_string());
    field("blue", &c.blue.to_string());
    println!();
}

fn print_detection_sensor(cfg: Option<&protobufs::module_config::DetectionSensorConfig>) {
    let Some(c) = cfg else {
        return not_received("Detection Sensor");
    };
    section_header("Detection Sensor");
    field("enabled", &c.enabled.to_string());
    field(
        "minimum_broadcast_secs",
        &c.minimum_broadcast_secs.to_string(),
    );
    field("state_broadcast_secs", &c.state_broadcast_secs.to_string());
    field("send_bell", &c.send_bell.to_string());
    field("name", &c.name);
    field("monitor_pin", &c.monitor_pin.to_string());
    field_enum("detection_trigger_type", c.detection_trigger_type, |v| {
        protobufs::module_config::detection_sensor_config::TriggerType::try_from(v).ok()
    });
    field("use_pullup", &c.use_pullup.to_string());
    println!();
}

fn print_paxcounter(cfg: Option<&protobufs::module_config::PaxcounterConfig>) {
    let Some(c) = cfg else {
        return not_received("Paxcounter");
    };
    section_header("Paxcounter");
    field("enabled", &c.enabled.to_string());
    field(
        "paxcounter_update_interval",
        &c.paxcounter_update_interval.to_string(),
    );
    field("wifi_threshold", &c.wifi_threshold.to_string());
    field("ble_threshold", &c.ble_threshold.to_string());
    println!();
}

// ── Field parsing helpers ──────────────────────────────────────────

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

fn parse_u32(value: &str) -> anyhow::Result<u32> {
    value
        .parse::<u32>()
        .map_err(|_| anyhow::anyhow!("Invalid u32 value '{}'", value))
}

fn parse_i32(value: &str) -> anyhow::Result<i32> {
    value
        .parse::<i32>()
        .map_err(|_| anyhow::anyhow!("Invalid i32 value '{}'", value))
}

fn parse_f32(value: &str) -> anyhow::Result<f32> {
    value
        .parse::<f32>()
        .map_err(|_| anyhow::anyhow!("Invalid f32 value '{}'", value))
}

fn parse_u64(value: &str) -> anyhow::Result<u64> {
    value
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("Invalid u64 value '{}'", value))
}

fn parse_enum_i32(value: &str, variants: &[(&str, i32)]) -> anyhow::Result<i32> {
    let upper = value.to_uppercase();
    for (name, val) in variants {
        if name.to_uppercase() == upper {
            return Ok(*val);
        }
    }
    // Also try parsing as a raw integer
    if let Ok(n) = value.parse::<i32>() {
        return Ok(n);
    }
    let names: Vec<&str> = variants.iter().map(|(n, _)| *n).collect();
    bail!(
        "Invalid value '{}'. Valid options: {}",
        value,
        names.join(", ")
    )
}

// ── Config field mutation ──────────────────────────────────────────

pub(super) fn apply_config_field(
    section: &str,
    field_name: &str,
    value: &str,
    config: &protobufs::LocalConfig,
) -> anyhow::Result<protobufs::config::PayloadVariant> {
    use protobufs::config::PayloadVariant;

    match section {
        "device" => {
            let mut c = config.device.clone().unwrap_or_default();
            match field_name {
                "role" => {
                    c.role = parse_enum_i32(
                        value,
                        &[
                            ("Client", 0),
                            ("ClientMute", 1),
                            ("Router", 2),
                            ("RouterClient", 3),
                            ("Repeater", 4),
                            ("Tracker", 5),
                            ("Sensor", 6),
                            ("Tak", 7),
                            ("ClientHidden", 8),
                            ("LostAndFound", 9),
                            ("TakTracker", 10),
                            ("RouterLate", 11),
                        ],
                    )?;
                }
                "node_info_broadcast_secs" => c.node_info_broadcast_secs = parse_u32(value)?,
                "double_tap_as_button_press" => c.double_tap_as_button_press = parse_bool(value)?,
                "disable_triple_click" => c.disable_triple_click = parse_bool(value)?,
                "led_heartbeat_disabled" => c.led_heartbeat_disabled = parse_bool(value)?,
                "button_gpio" => c.button_gpio = parse_u32(value)?,
                "buzzer_gpio" => c.buzzer_gpio = parse_u32(value)?,
                "rebroadcast_mode" => {
                    c.rebroadcast_mode = parse_enum_i32(
                        value,
                        &[
                            ("All", 0),
                            ("AllSkipDecoding", 1),
                            ("LocalOnly", 2),
                            ("KnownOnly", 3),
                            ("None", 4),
                            ("CorePortnumsOnly", 5),
                        ],
                    )?;
                }
                "buzzer_mode" => {
                    c.buzzer_mode = parse_enum_i32(
                        value,
                        &[
                            ("AllEnabled", 0),
                            ("Disabled", 1),
                            ("NotificationsOnly", 2),
                            ("SystemOnly", 3),
                            ("DirectMsgOnly", 4),
                        ],
                    )?;
                }
                "tzdef" => c.tzdef = value.to_string(),
                _ => bail!("Unknown field 'device.{}'", field_name),
            }
            Ok(PayloadVariant::Device(c))
        }
        "position" => {
            let mut c = config.position.unwrap_or_default();
            match field_name {
                "position_broadcast_secs" => c.position_broadcast_secs = parse_u32(value)?,
                "position_broadcast_smart_enabled" => {
                    c.position_broadcast_smart_enabled = parse_bool(value)?;
                }
                "fixed_position" => c.fixed_position = parse_bool(value)?,
                "gps_update_interval" => c.gps_update_interval = parse_u32(value)?,
                "position_flags" => c.position_flags = parse_u32(value)?,
                "rx_gpio" => c.rx_gpio = parse_u32(value)?,
                "tx_gpio" => c.tx_gpio = parse_u32(value)?,
                "broadcast_smart_minimum_distance" => {
                    c.broadcast_smart_minimum_distance = parse_u32(value)?;
                }
                "broadcast_smart_minimum_interval_secs" => {
                    c.broadcast_smart_minimum_interval_secs = parse_u32(value)?;
                }
                "gps_en_gpio" => c.gps_en_gpio = parse_u32(value)?,
                "gps_mode" => {
                    c.gps_mode = parse_enum_i32(
                        value,
                        &[("Disabled", 0), ("Enabled", 1), ("NotPresent", 2)],
                    )?;
                }
                _ => bail!("Unknown field 'position.{}'", field_name),
            }
            Ok(PayloadVariant::Position(c))
        }
        "power" => {
            let mut c = config.power.unwrap_or_default();
            match field_name {
                "is_power_saving" => c.is_power_saving = parse_bool(value)?,
                "on_battery_shutdown_after_secs" => {
                    c.on_battery_shutdown_after_secs = parse_u32(value)?;
                }
                "adc_multiplier_override" => c.adc_multiplier_override = parse_f32(value)?,
                "wait_bluetooth_secs" => c.wait_bluetooth_secs = parse_u32(value)?,
                "sds_secs" => c.sds_secs = parse_u32(value)?,
                "ls_secs" => c.ls_secs = parse_u32(value)?,
                "min_wake_secs" => c.min_wake_secs = parse_u32(value)?,
                "device_battery_ina_address" => c.device_battery_ina_address = parse_u32(value)?,
                "powermon_enables" => c.powermon_enables = parse_u64(value)?,
                _ => bail!("Unknown field 'power.{}'", field_name),
            }
            Ok(PayloadVariant::Power(c))
        }
        "network" => {
            let mut c = config.network.clone().unwrap_or_default();
            match field_name {
                "wifi_enabled" => c.wifi_enabled = parse_bool(value)?,
                "wifi_ssid" => c.wifi_ssid = value.to_string(),
                "wifi_psk" => c.wifi_psk = value.to_string(),
                "ntp_server" => c.ntp_server = value.to_string(),
                "eth_enabled" => c.eth_enabled = parse_bool(value)?,
                "address_mode" => {
                    c.address_mode = parse_enum_i32(value, &[("Dhcp", 0), ("Static", 1)])?;
                }
                "rsyslog_server" => c.rsyslog_server = value.to_string(),
                "ipv6_enabled" => c.ipv6_enabled = parse_bool(value)?,
                _ => bail!("Unknown field 'network.{}'", field_name),
            }
            Ok(PayloadVariant::Network(c))
        }
        "display" => {
            let mut c = config.display.unwrap_or_default();
            match field_name {
                "screen_on_secs" => c.screen_on_secs = parse_u32(value)?,
                "auto_screen_carousel_secs" => c.auto_screen_carousel_secs = parse_u32(value)?,
                "flip_screen" => c.flip_screen = parse_bool(value)?,
                "units" => {
                    c.units = parse_enum_i32(value, &[("Metric", 0), ("Imperial", 1)])?;
                }
                "oled" => {
                    c.oled = parse_enum_i32(
                        value,
                        &[("Auto", 0), ("Ssd1306", 1), ("Sh1106", 2), ("Sh1107", 3)],
                    )?;
                }
                "displaymode" => {
                    c.displaymode = parse_enum_i32(
                        value,
                        &[
                            ("Default", 0),
                            ("TwoColor", 1),
                            ("Inverted", 2),
                            ("Color", 3),
                        ],
                    )?;
                }
                "heading_bold" => c.heading_bold = parse_bool(value)?,
                "wake_on_tap_or_motion" => c.wake_on_tap_or_motion = parse_bool(value)?,
                "compass_orientation" => {
                    c.compass_orientation = parse_enum_i32(
                        value,
                        &[
                            ("DegreesDefault", 0),
                            ("Degrees0", 0),
                            ("Degrees90", 1),
                            ("Degrees180", 2),
                            ("Degrees270", 3),
                        ],
                    )?;
                }
                "use_12h_clock" => c.use_12h_clock = parse_bool(value)?,
                _ => bail!("Unknown field 'display.{}'", field_name),
            }
            Ok(PayloadVariant::Display(c))
        }
        "lora" => {
            let mut c = config.lora.clone().unwrap_or_default();
            match field_name {
                "use_preset" => c.use_preset = parse_bool(value)?,
                "modem_preset" => {
                    c.modem_preset = parse_enum_i32(
                        value,
                        &[
                            ("LongFast", 0),
                            ("LongSlow", 1),
                            ("VeryLongSlow", 2),
                            ("MediumSlow", 3),
                            ("MediumFast", 4),
                            ("ShortSlow", 5),
                            ("ShortFast", 6),
                            ("LongModerate", 7),
                            ("ShortTurbo", 8),
                        ],
                    )?;
                }
                "bandwidth" => c.bandwidth = parse_u32(value)?,
                "spread_factor" => c.spread_factor = parse_u32(value)?,
                "coding_rate" => c.coding_rate = parse_u32(value)?,
                "frequency_offset" => c.frequency_offset = parse_f32(value)?,
                "region" => {
                    c.region = parse_enum_i32(
                        value,
                        &[
                            ("Unset", 0),
                            ("Us", 1),
                            ("Eu433", 2),
                            ("Eu868", 3),
                            ("Cn", 4),
                            ("Jp", 5),
                            ("Anz", 6),
                            ("Kr", 7),
                            ("Tw", 8),
                            ("Ru", 9),
                            ("In", 10),
                            ("Nz865", 11),
                            ("Th", 12),
                            ("Lora24", 13),
                            ("Ua433", 14),
                            ("Ua868", 15),
                            ("My433", 16),
                            ("My919", 17),
                            ("Sg923", 18),
                            ("Ph433", 19),
                            ("Ph868", 20),
                            ("Ph915", 21),
                            ("Anz433", 22),
                            ("Kz433", 23),
                            ("Kz863", 24),
                            ("Np865", 25),
                            ("Br902", 26),
                        ],
                    )?;
                }
                "hop_limit" => c.hop_limit = parse_u32(value)?,
                "tx_enabled" => c.tx_enabled = parse_bool(value)?,
                "tx_power" => c.tx_power = parse_i32(value)?,
                "channel_num" => c.channel_num = parse_u32(value)?,
                "override_duty_cycle" => c.override_duty_cycle = parse_bool(value)?,
                "sx126x_rx_boosted_gain" => c.sx126x_rx_boosted_gain = parse_bool(value)?,
                "override_frequency" => c.override_frequency = parse_f32(value)?,
                "pa_fan_disabled" => c.pa_fan_disabled = parse_bool(value)?,
                "ignore_mqtt" => c.ignore_mqtt = parse_bool(value)?,
                "config_ok_to_mqtt" => c.config_ok_to_mqtt = parse_bool(value)?,
                _ => bail!("Unknown field 'lora.{}'", field_name),
            }
            Ok(PayloadVariant::Lora(c))
        }
        "bluetooth" => {
            let mut c = config.bluetooth.unwrap_or_default();
            match field_name {
                "enabled" => c.enabled = parse_bool(value)?,
                "mode" => {
                    c.mode =
                        parse_enum_i32(value, &[("RandomPin", 0), ("FixedPin", 1), ("NoPin", 2)])?;
                }
                "fixed_pin" => c.fixed_pin = parse_u32(value)?,
                _ => bail!("Unknown field 'bluetooth.{}'", field_name),
            }
            Ok(PayloadVariant::Bluetooth(c))
        }
        "security" => {
            let mut c = config.security.clone().unwrap_or_default();
            match field_name {
                "is_managed" => c.is_managed = parse_bool(value)?,
                "serial_enabled" => c.serial_enabled = parse_bool(value)?,
                "debug_log_api_enabled" => c.debug_log_api_enabled = parse_bool(value)?,
                "admin_channel_enabled" => c.admin_channel_enabled = parse_bool(value)?,
                _ => bail!("Unknown field 'security.{}'", field_name),
            }
            Ok(PayloadVariant::Security(c))
        }
        _ => bail!("Unknown config section '{}'", section),
    }
}

pub(super) fn apply_module_config_field(
    section: &str,
    field_name: &str,
    value: &str,
    module: &protobufs::LocalModuleConfig,
) -> anyhow::Result<protobufs::module_config::PayloadVariant> {
    use protobufs::module_config::PayloadVariant;

    match section {
        "mqtt" => {
            let mut c = module.mqtt.clone().unwrap_or_default();
            match field_name {
                "enabled" => c.enabled = parse_bool(value)?,
                "address" => c.address = value.to_string(),
                "username" => c.username = value.to_string(),
                "password" => c.password = value.to_string(),
                "encryption_enabled" => c.encryption_enabled = parse_bool(value)?,
                "json_enabled" => c.json_enabled = parse_bool(value)?,
                "tls_enabled" => c.tls_enabled = parse_bool(value)?,
                "root" => c.root = value.to_string(),
                "proxy_to_client_enabled" => c.proxy_to_client_enabled = parse_bool(value)?,
                "map_reporting_enabled" => c.map_reporting_enabled = parse_bool(value)?,
                _ => bail!("Unknown field 'mqtt.{}'", field_name),
            }
            Ok(PayloadVariant::Mqtt(c))
        }
        "serial" => {
            let mut c = module.serial.unwrap_or_default();
            match field_name {
                "enabled" => c.enabled = parse_bool(value)?,
                "echo" => c.echo = parse_bool(value)?,
                "rxd" => c.rxd = parse_u32(value)?,
                "txd" => c.txd = parse_u32(value)?,
                "baud" => {
                    c.baud = parse_enum_i32(
                        value,
                        &[
                            ("Default", 0),
                            ("Baud110", 1),
                            ("Baud300", 2),
                            ("Baud600", 3),
                            ("Baud1200", 4),
                            ("Baud2400", 5),
                            ("Baud4800", 6),
                            ("Baud9600", 7),
                            ("Baud19200", 8),
                            ("Baud38400", 9),
                            ("Baud57600", 10),
                            ("Baud115200", 11),
                            ("Baud230400", 12),
                            ("Baud460800", 13),
                            ("Baud576000", 14),
                            ("Baud921600", 15),
                        ],
                    )?;
                }
                "timeout" => c.timeout = parse_u32(value)?,
                "mode" => {
                    c.mode = parse_enum_i32(
                        value,
                        &[
                            ("Default", 0),
                            ("Simple", 1),
                            ("Proto", 2),
                            ("TextMsg", 3),
                            ("Nmea", 4),
                            ("CalTopo", 5),
                        ],
                    )?;
                }
                "override_console_serial_port" => {
                    c.override_console_serial_port = parse_bool(value)?;
                }
                _ => bail!("Unknown field 'serial.{}'", field_name),
            }
            Ok(PayloadVariant::Serial(c))
        }
        "external_notification" => {
            let mut c = module.external_notification.unwrap_or_default();
            match field_name {
                "enabled" => c.enabled = parse_bool(value)?,
                "output_ms" => c.output_ms = parse_u32(value)?,
                "output" => c.output = parse_u32(value)?,
                "output_vibra" => c.output_vibra = parse_u32(value)?,
                "output_buzzer" => c.output_buzzer = parse_u32(value)?,
                "active" => c.active = parse_bool(value)?,
                "alert_message" => c.alert_message = parse_bool(value)?,
                "alert_message_vibra" => c.alert_message_vibra = parse_bool(value)?,
                "alert_message_buzzer" => c.alert_message_buzzer = parse_bool(value)?,
                "alert_bell" => c.alert_bell = parse_bool(value)?,
                "alert_bell_vibra" => c.alert_bell_vibra = parse_bool(value)?,
                "alert_bell_buzzer" => c.alert_bell_buzzer = parse_bool(value)?,
                "use_pwm" => c.use_pwm = parse_bool(value)?,
                "nag_timeout" => c.nag_timeout = parse_u32(value)?,
                "use_i2s_as_buzzer" => c.use_i2s_as_buzzer = parse_bool(value)?,
                _ => bail!("Unknown field 'external_notification.{}'", field_name),
            }
            Ok(PayloadVariant::ExternalNotification(c))
        }
        "store_forward" => {
            let mut c = module.store_forward.unwrap_or_default();
            match field_name {
                "enabled" => c.enabled = parse_bool(value)?,
                "heartbeat" => c.heartbeat = parse_bool(value)?,
                "records" => c.records = parse_u32(value)?,
                "history_return_max" => c.history_return_max = parse_u32(value)?,
                "history_return_window" => c.history_return_window = parse_u32(value)?,
                "is_server" => c.is_server = parse_bool(value)?,
                _ => bail!("Unknown field 'store_forward.{}'", field_name),
            }
            Ok(PayloadVariant::StoreForward(c))
        }
        "range_test" => {
            let mut c = module.range_test.unwrap_or_default();
            match field_name {
                "enabled" => c.enabled = parse_bool(value)?,
                "sender" => c.sender = parse_u32(value)?,
                "save" => c.save = parse_bool(value)?,
                _ => bail!("Unknown field 'range_test.{}'", field_name),
            }
            Ok(PayloadVariant::RangeTest(c))
        }
        "telemetry" => {
            let mut c = module.telemetry.unwrap_or_default();
            match field_name {
                "device_update_interval" => c.device_update_interval = parse_u32(value)?,
                "environment_update_interval" => {
                    c.environment_update_interval = parse_u32(value)?;
                }
                "environment_measurement_enabled" => {
                    c.environment_measurement_enabled = parse_bool(value)?;
                }
                "environment_screen_enabled" => {
                    c.environment_screen_enabled = parse_bool(value)?;
                }
                "environment_display_fahrenheit" => {
                    c.environment_display_fahrenheit = parse_bool(value)?;
                }
                "air_quality_enabled" => c.air_quality_enabled = parse_bool(value)?,
                "air_quality_interval" => c.air_quality_interval = parse_u32(value)?,
                "power_measurement_enabled" => {
                    c.power_measurement_enabled = parse_bool(value)?;
                }
                "power_update_interval" => c.power_update_interval = parse_u32(value)?,
                "power_screen_enabled" => c.power_screen_enabled = parse_bool(value)?,
                "health_measurement_enabled" => {
                    c.health_measurement_enabled = parse_bool(value)?;
                }
                "health_update_interval" => c.health_update_interval = parse_u32(value)?,
                "health_screen_enabled" => c.health_screen_enabled = parse_bool(value)?,
                _ => bail!("Unknown field 'telemetry.{}'", field_name),
            }
            Ok(PayloadVariant::Telemetry(c))
        }
        "canned_message" => {
            let mut c = module.canned_message.clone().unwrap_or_default();
            match field_name {
                "rotary1_enabled" => c.rotary1_enabled = parse_bool(value)?,
                "inputbroker_pin_a" => c.inputbroker_pin_a = parse_u32(value)?,
                "inputbroker_pin_b" => c.inputbroker_pin_b = parse_u32(value)?,
                "inputbroker_pin_press" => c.inputbroker_pin_press = parse_u32(value)?,
                "inputbroker_event_cw" => {
                    c.inputbroker_event_cw = parse_enum_i32(
                        value,
                        &[
                            ("None", 0),
                            ("Up", 17),
                            ("Down", 18),
                            ("Left", 19),
                            ("Right", 20),
                            ("Select", 10),
                            ("Back", 27),
                            ("Cancel", 24),
                        ],
                    )?;
                }
                "inputbroker_event_ccw" => {
                    c.inputbroker_event_ccw = parse_enum_i32(
                        value,
                        &[
                            ("None", 0),
                            ("Up", 17),
                            ("Down", 18),
                            ("Left", 19),
                            ("Right", 20),
                            ("Select", 10),
                            ("Back", 27),
                            ("Cancel", 24),
                        ],
                    )?;
                }
                "inputbroker_event_press" => {
                    c.inputbroker_event_press = parse_enum_i32(
                        value,
                        &[
                            ("None", 0),
                            ("Up", 17),
                            ("Down", 18),
                            ("Left", 19),
                            ("Right", 20),
                            ("Select", 10),
                            ("Back", 27),
                            ("Cancel", 24),
                        ],
                    )?;
                }
                "updown1_enabled" => c.updown1_enabled = parse_bool(value)?,
                "send_bell" => c.send_bell = parse_bool(value)?,
                _ => bail!("Unknown field 'canned_message.{}'", field_name),
            }
            Ok(PayloadVariant::CannedMessage(c))
        }
        "audio" => {
            let mut c = module.audio.unwrap_or_default();
            match field_name {
                "codec2_enabled" => c.codec2_enabled = parse_bool(value)?,
                "ptt_pin" => c.ptt_pin = parse_u32(value)?,
                "bitrate" => {
                    c.bitrate = parse_enum_i32(
                        value,
                        &[
                            ("Codec2Default", 0),
                            ("Codec2_3200", 1),
                            ("Codec2_2400", 2),
                            ("Codec2_1600", 3),
                            ("Codec2_1400", 4),
                            ("Codec2_1300", 5),
                            ("Codec2_1200", 6),
                            ("Codec2_700", 7),
                            ("Codec2_700B", 8),
                        ],
                    )?;
                }
                "i2s_ws" => c.i2s_ws = parse_u32(value)?,
                "i2s_sd" => c.i2s_sd = parse_u32(value)?,
                "i2s_din" => c.i2s_din = parse_u32(value)?,
                "i2s_sck" => c.i2s_sck = parse_u32(value)?,
                _ => bail!("Unknown field 'audio.{}'", field_name),
            }
            Ok(PayloadVariant::Audio(c))
        }
        "remote_hardware" => {
            let mut c = module.remote_hardware.clone().unwrap_or_default();
            match field_name {
                "enabled" => c.enabled = parse_bool(value)?,
                "allow_undefined_pin_access" => {
                    c.allow_undefined_pin_access = parse_bool(value)?;
                }
                _ => bail!("Unknown field 'remote_hardware.{}'", field_name),
            }
            Ok(PayloadVariant::RemoteHardware(c))
        }
        "neighbor_info" => {
            let mut c = module.neighbor_info.unwrap_or_default();
            match field_name {
                "enabled" => c.enabled = parse_bool(value)?,
                "update_interval" => c.update_interval = parse_u32(value)?,
                "transmit_over_lora" => c.transmit_over_lora = parse_bool(value)?,
                _ => bail!("Unknown field 'neighbor_info.{}'", field_name),
            }
            Ok(PayloadVariant::NeighborInfo(c))
        }
        "ambient_lighting" => {
            let mut c = module.ambient_lighting.unwrap_or_default();
            match field_name {
                "led_state" => c.led_state = parse_bool(value)?,
                "current" => c.current = parse_u32(value)?,
                "red" => c.red = parse_u32(value)?,
                "green" => c.green = parse_u32(value)?,
                "blue" => c.blue = parse_u32(value)?,
                _ => bail!("Unknown field 'ambient_lighting.{}'", field_name),
            }
            Ok(PayloadVariant::AmbientLighting(c))
        }
        "detection_sensor" => {
            let mut c = module.detection_sensor.clone().unwrap_or_default();
            match field_name {
                "enabled" => c.enabled = parse_bool(value)?,
                "minimum_broadcast_secs" => c.minimum_broadcast_secs = parse_u32(value)?,
                "state_broadcast_secs" => c.state_broadcast_secs = parse_u32(value)?,
                "send_bell" => c.send_bell = parse_bool(value)?,
                "name" => c.name = value.to_string(),
                "monitor_pin" => c.monitor_pin = parse_u32(value)?,
                "detection_trigger_type" => {
                    c.detection_trigger_type = parse_enum_i32(
                        value,
                        &[
                            ("Logic", 0),
                            ("FallingEdge", 1),
                            ("RisingEdge", 2),
                            ("EitherEdge", 3),
                        ],
                    )?;
                }
                "use_pullup" => c.use_pullup = parse_bool(value)?,
                _ => bail!("Unknown field 'detection_sensor.{}'", field_name),
            }
            Ok(PayloadVariant::DetectionSensor(c))
        }
        "paxcounter" => {
            let mut c = module.paxcounter.unwrap_or_default();
            match field_name {
                "enabled" => c.enabled = parse_bool(value)?,
                "paxcounter_update_interval" => {
                    c.paxcounter_update_interval = parse_u32(value)?;
                }
                "wifi_threshold" => c.wifi_threshold = parse_i32(value)?,
                "ble_threshold" => c.ble_threshold = parse_i32(value)?,
                _ => bail!("Unknown field 'paxcounter.{}'", field_name),
            }
            Ok(PayloadVariant::Paxcounter(c))
        }
        _ => bail!("Unknown module config section '{}'", section),
    }
}
