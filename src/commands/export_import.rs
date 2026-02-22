use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::bail;
use async_trait::async_trait;
use colored::Colorize;
use meshtastic::protobufs::{self, channel, ChannelSettings};
use serde_yaml::Value;

use super::{Command, CommandContext};

// ── ExportConfigCommand ────────────────────────────────────────────

pub struct ExportConfigCommand {
    pub file: Option<PathBuf>,
}

#[async_trait]
impl Command for ExportConfigCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let config = ctx.node_db.local_config();
        let module = ctx.node_db.local_module_config();
        let channels = ctx.node_db.channels();

        let mut root = BTreeMap::new();

        // Config sections
        if let Some(c) = &config.device {
            root.insert("device".to_string(), export_device(c));
        }
        if let Some(c) = &config.position {
            root.insert("position".to_string(), export_position(c));
        }
        if let Some(c) = &config.power {
            root.insert("power".to_string(), export_power(c));
        }
        if let Some(c) = &config.network {
            root.insert("network".to_string(), export_network(c));
        }
        if let Some(c) = &config.display {
            root.insert("display".to_string(), export_display(c));
        }
        if let Some(c) = &config.lora {
            root.insert("lora".to_string(), export_lora(c));
        }
        if let Some(c) = &config.bluetooth {
            root.insert("bluetooth".to_string(), export_bluetooth(c));
        }
        if let Some(c) = &config.security {
            root.insert("security".to_string(), export_security(c));
        }

        // Module config sections
        if let Some(c) = &module.mqtt {
            root.insert("mqtt".to_string(), export_mqtt(c));
        }
        if let Some(c) = &module.serial {
            root.insert("serial".to_string(), export_serial(c));
        }
        if let Some(c) = &module.external_notification {
            root.insert(
                "external_notification".to_string(),
                export_external_notification(c),
            );
        }
        if let Some(c) = &module.store_forward {
            root.insert("store_forward".to_string(), export_store_forward(c));
        }
        if let Some(c) = &module.range_test {
            root.insert("range_test".to_string(), export_range_test(c));
        }
        if let Some(c) = &module.telemetry {
            root.insert("telemetry".to_string(), export_telemetry(c));
        }
        if let Some(c) = &module.canned_message {
            root.insert("canned_message".to_string(), export_canned_message(c));
        }
        if let Some(c) = &module.audio {
            root.insert("audio".to_string(), export_audio(c));
        }
        if let Some(c) = &module.remote_hardware {
            root.insert("remote_hardware".to_string(), export_remote_hardware(c));
        }
        if let Some(c) = &module.neighbor_info {
            root.insert("neighbor_info".to_string(), export_neighbor_info(c));
        }
        if let Some(c) = &module.ambient_lighting {
            root.insert("ambient_lighting".to_string(), export_ambient_lighting(c));
        }
        if let Some(c) = &module.detection_sensor {
            root.insert("detection_sensor".to_string(), export_detection_sensor(c));
        }
        if let Some(c) = &module.paxcounter {
            root.insert("paxcounter".to_string(), export_paxcounter(c));
        }

        // Channels
        let active_channels: Vec<_> = channels
            .iter()
            .filter(|c| c.role != channel::Role::Disabled as i32)
            .collect();

        if !active_channels.is_empty() {
            let ch_list: Vec<Value> = active_channels.iter().map(|c| export_channel(c)).collect();
            root.insert("channels".to_string(), Value::Sequence(ch_list));
        }

        let yaml = serde_yaml::to_string(&root)?;

        match &self.file {
            Some(path) => {
                std::fs::write(path, &yaml)?;
                println!(
                    "{} Configuration exported to {}",
                    "ok".green(),
                    path.display().to_string().bold()
                );
            }
            None => {
                print!("{}", yaml);
            }
        }

        Ok(())
    }
}

// ── ImportConfigCommand ────────────────────────────────────────────

pub struct ImportConfigCommand {
    pub file: PathBuf,
}

#[async_trait]
impl Command for ImportConfigCommand {
    async fn execute(&self, ctx: &mut CommandContext) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(&self.file)?;
        let root: BTreeMap<String, Value> = serde_yaml::from_str(&content)?;

        println!(
            "{} Importing configuration from {}...",
            "->".cyan(),
            self.file.display().to_string().bold()
        );

        let config = ctx.node_db.local_config().clone();
        let module = ctx.node_db.local_module_config().clone();

        let mut config_count = 0u32;
        let mut module_count = 0u32;
        let mut channel_count = 0u32;

        for (section, values) in &root {
            if section == "channels" {
                let channels = import_channels(values)?;
                for ch in channels {
                    ctx.api.update_channel_config(&mut ctx.router, ch).await?;
                    channel_count += 1;
                }
                continue;
            }

            let Value::Mapping(fields) = values else {
                bail!("Section '{}' must be a mapping", section);
            };

            match section.as_str() {
                "device" | "position" | "power" | "network" | "display" | "lora" | "bluetooth"
                | "security" => {
                    let payload = import_config_section(section, fields, &config)?;
                    let config_packet = protobufs::Config {
                        payload_variant: Some(payload),
                    };
                    ctx.api
                        .update_config(&mut ctx.router, config_packet)
                        .await?;
                    config_count += 1;
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
                    let payload = import_module_config_section(section, fields, &module)?;
                    let module_packet = protobufs::ModuleConfig {
                        payload_variant: Some(payload),
                    };
                    ctx.api
                        .update_module_config(&mut ctx.router, module_packet)
                        .await?;
                    module_count += 1;
                }
                _ => {
                    println!("  {} Skipping unknown section '{}'", "!".yellow(), section);
                }
            }
        }

        println!(
            "{} Imported {} config sections, {} module sections, {} channels.",
            "ok".green(),
            config_count,
            module_count,
            channel_count
        );
        if config_count > 0 || module_count > 0 {
            println!(
                "{} Device will reboot to apply configuration changes.",
                "!".yellow().bold()
            );
        }

        Ok(())
    }
}

// ── Export helpers ──────────────────────────────────────────────────

fn val_str(s: &str) -> Value {
    Value::String(s.to_string())
}

fn val_bool(b: bool) -> Value {
    Value::Bool(b)
}

fn val_u32(n: u32) -> Value {
    Value::Number(serde_yaml::Number::from(n))
}

fn val_i32(n: i32) -> Value {
    Value::Number(serde_yaml::Number::from(n))
}

fn val_f32(n: f32) -> Value {
    serde_yaml::to_value(n).unwrap_or(Value::Null)
}

fn val_u64(n: u64) -> Value {
    Value::Number(serde_yaml::Number::from(n))
}

fn map_from(pairs: Vec<(&str, Value)>) -> Value {
    let mut m = serde_yaml::Mapping::new();
    for (k, v) in pairs {
        m.insert(Value::String(k.to_string()), v);
    }
    Value::Mapping(m)
}

fn export_device(c: &protobufs::config::DeviceConfig) -> Value {
    map_from(vec![
        ("role", val_i32(c.role)),
        (
            "node_info_broadcast_secs",
            val_u32(c.node_info_broadcast_secs),
        ),
        (
            "double_tap_as_button_press",
            val_bool(c.double_tap_as_button_press),
        ),
        ("disable_triple_click", val_bool(c.disable_triple_click)),
        ("led_heartbeat_disabled", val_bool(c.led_heartbeat_disabled)),
        ("button_gpio", val_u32(c.button_gpio)),
        ("buzzer_gpio", val_u32(c.buzzer_gpio)),
        ("rebroadcast_mode", val_i32(c.rebroadcast_mode)),
        ("buzzer_mode", val_i32(c.buzzer_mode)),
        ("tzdef", val_str(&c.tzdef)),
    ])
}

fn export_position(c: &protobufs::config::PositionConfig) -> Value {
    map_from(vec![
        (
            "position_broadcast_secs",
            val_u32(c.position_broadcast_secs),
        ),
        (
            "position_broadcast_smart_enabled",
            val_bool(c.position_broadcast_smart_enabled),
        ),
        ("fixed_position", val_bool(c.fixed_position)),
        ("gps_update_interval", val_u32(c.gps_update_interval)),
        ("position_flags", val_u32(c.position_flags)),
        ("rx_gpio", val_u32(c.rx_gpio)),
        ("tx_gpio", val_u32(c.tx_gpio)),
        (
            "broadcast_smart_minimum_distance",
            val_u32(c.broadcast_smart_minimum_distance),
        ),
        (
            "broadcast_smart_minimum_interval_secs",
            val_u32(c.broadcast_smart_minimum_interval_secs),
        ),
        ("gps_en_gpio", val_u32(c.gps_en_gpio)),
        ("gps_mode", val_i32(c.gps_mode)),
    ])
}

fn export_power(c: &protobufs::config::PowerConfig) -> Value {
    map_from(vec![
        ("is_power_saving", val_bool(c.is_power_saving)),
        (
            "on_battery_shutdown_after_secs",
            val_u32(c.on_battery_shutdown_after_secs),
        ),
        (
            "adc_multiplier_override",
            val_f32(c.adc_multiplier_override),
        ),
        ("wait_bluetooth_secs", val_u32(c.wait_bluetooth_secs)),
        ("sds_secs", val_u32(c.sds_secs)),
        ("ls_secs", val_u32(c.ls_secs)),
        ("min_wake_secs", val_u32(c.min_wake_secs)),
        (
            "device_battery_ina_address",
            val_u32(c.device_battery_ina_address),
        ),
        ("powermon_enables", val_u64(c.powermon_enables)),
    ])
}

fn export_network(c: &protobufs::config::NetworkConfig) -> Value {
    map_from(vec![
        ("wifi_enabled", val_bool(c.wifi_enabled)),
        ("wifi_ssid", val_str(&c.wifi_ssid)),
        ("wifi_psk", val_str(&c.wifi_psk)),
        ("ntp_server", val_str(&c.ntp_server)),
        ("eth_enabled", val_bool(c.eth_enabled)),
        ("address_mode", val_i32(c.address_mode)),
        ("rsyslog_server", val_str(&c.rsyslog_server)),
        ("ipv6_enabled", val_bool(c.ipv6_enabled)),
    ])
}

fn export_display(c: &protobufs::config::DisplayConfig) -> Value {
    map_from(vec![
        ("screen_on_secs", val_u32(c.screen_on_secs)),
        (
            "auto_screen_carousel_secs",
            val_u32(c.auto_screen_carousel_secs),
        ),
        ("flip_screen", val_bool(c.flip_screen)),
        ("units", val_i32(c.units)),
        ("oled", val_i32(c.oled)),
        ("displaymode", val_i32(c.displaymode)),
        ("heading_bold", val_bool(c.heading_bold)),
        ("wake_on_tap_or_motion", val_bool(c.wake_on_tap_or_motion)),
        ("compass_orientation", val_i32(c.compass_orientation)),
        ("use_12h_clock", val_bool(c.use_12h_clock)),
    ])
}

fn export_lora(c: &protobufs::config::LoRaConfig) -> Value {
    map_from(vec![
        ("use_preset", val_bool(c.use_preset)),
        ("modem_preset", val_i32(c.modem_preset)),
        ("bandwidth", val_u32(c.bandwidth)),
        ("spread_factor", val_u32(c.spread_factor)),
        ("coding_rate", val_u32(c.coding_rate)),
        ("frequency_offset", val_f32(c.frequency_offset)),
        ("region", val_i32(c.region)),
        ("hop_limit", val_u32(c.hop_limit)),
        ("tx_enabled", val_bool(c.tx_enabled)),
        ("tx_power", val_i32(c.tx_power)),
        ("channel_num", val_u32(c.channel_num)),
        ("override_duty_cycle", val_bool(c.override_duty_cycle)),
        ("sx126x_rx_boosted_gain", val_bool(c.sx126x_rx_boosted_gain)),
        ("override_frequency", val_f32(c.override_frequency)),
        ("pa_fan_disabled", val_bool(c.pa_fan_disabled)),
        ("ignore_mqtt", val_bool(c.ignore_mqtt)),
        ("config_ok_to_mqtt", val_bool(c.config_ok_to_mqtt)),
    ])
}

fn export_bluetooth(c: &protobufs::config::BluetoothConfig) -> Value {
    map_from(vec![
        ("enabled", val_bool(c.enabled)),
        ("mode", val_i32(c.mode)),
        ("fixed_pin", val_u32(c.fixed_pin)),
    ])
}

fn export_security(c: &protobufs::config::SecurityConfig) -> Value {
    map_from(vec![
        ("is_managed", val_bool(c.is_managed)),
        ("serial_enabled", val_bool(c.serial_enabled)),
        ("debug_log_api_enabled", val_bool(c.debug_log_api_enabled)),
        ("admin_channel_enabled", val_bool(c.admin_channel_enabled)),
    ])
}

fn export_mqtt(c: &protobufs::module_config::MqttConfig) -> Value {
    map_from(vec![
        ("enabled", val_bool(c.enabled)),
        ("address", val_str(&c.address)),
        ("username", val_str(&c.username)),
        ("password", val_str(&c.password)),
        ("encryption_enabled", val_bool(c.encryption_enabled)),
        ("json_enabled", val_bool(c.json_enabled)),
        ("tls_enabled", val_bool(c.tls_enabled)),
        ("root", val_str(&c.root)),
        (
            "proxy_to_client_enabled",
            val_bool(c.proxy_to_client_enabled),
        ),
        ("map_reporting_enabled", val_bool(c.map_reporting_enabled)),
    ])
}

fn export_serial(c: &protobufs::module_config::SerialConfig) -> Value {
    map_from(vec![
        ("enabled", val_bool(c.enabled)),
        ("echo", val_bool(c.echo)),
        ("rxd", val_u32(c.rxd)),
        ("txd", val_u32(c.txd)),
        ("baud", val_i32(c.baud)),
        ("timeout", val_u32(c.timeout)),
        ("mode", val_i32(c.mode)),
        (
            "override_console_serial_port",
            val_bool(c.override_console_serial_port),
        ),
    ])
}

fn export_external_notification(c: &protobufs::module_config::ExternalNotificationConfig) -> Value {
    map_from(vec![
        ("enabled", val_bool(c.enabled)),
        ("output_ms", val_u32(c.output_ms)),
        ("output", val_u32(c.output)),
        ("output_vibra", val_u32(c.output_vibra)),
        ("output_buzzer", val_u32(c.output_buzzer)),
        ("active", val_bool(c.active)),
        ("alert_message", val_bool(c.alert_message)),
        ("alert_message_vibra", val_bool(c.alert_message_vibra)),
        ("alert_message_buzzer", val_bool(c.alert_message_buzzer)),
        ("alert_bell", val_bool(c.alert_bell)),
        ("alert_bell_vibra", val_bool(c.alert_bell_vibra)),
        ("alert_bell_buzzer", val_bool(c.alert_bell_buzzer)),
        ("use_pwm", val_bool(c.use_pwm)),
        ("nag_timeout", val_u32(c.nag_timeout)),
        ("use_i2s_as_buzzer", val_bool(c.use_i2s_as_buzzer)),
    ])
}

fn export_store_forward(c: &protobufs::module_config::StoreForwardConfig) -> Value {
    map_from(vec![
        ("enabled", val_bool(c.enabled)),
        ("heartbeat", val_bool(c.heartbeat)),
        ("records", val_u32(c.records)),
        ("history_return_max", val_u32(c.history_return_max)),
        ("history_return_window", val_u32(c.history_return_window)),
        ("is_server", val_bool(c.is_server)),
    ])
}

fn export_range_test(c: &protobufs::module_config::RangeTestConfig) -> Value {
    map_from(vec![
        ("enabled", val_bool(c.enabled)),
        ("sender", val_u32(c.sender)),
        ("save", val_bool(c.save)),
    ])
}

fn export_telemetry(c: &protobufs::module_config::TelemetryConfig) -> Value {
    map_from(vec![
        ("device_update_interval", val_u32(c.device_update_interval)),
        (
            "environment_update_interval",
            val_u32(c.environment_update_interval),
        ),
        (
            "environment_measurement_enabled",
            val_bool(c.environment_measurement_enabled),
        ),
        (
            "environment_screen_enabled",
            val_bool(c.environment_screen_enabled),
        ),
        (
            "environment_display_fahrenheit",
            val_bool(c.environment_display_fahrenheit),
        ),
        ("air_quality_enabled", val_bool(c.air_quality_enabled)),
        ("air_quality_interval", val_u32(c.air_quality_interval)),
        (
            "power_measurement_enabled",
            val_bool(c.power_measurement_enabled),
        ),
        ("power_update_interval", val_u32(c.power_update_interval)),
        ("power_screen_enabled", val_bool(c.power_screen_enabled)),
        (
            "health_measurement_enabled",
            val_bool(c.health_measurement_enabled),
        ),
        ("health_update_interval", val_u32(c.health_update_interval)),
        ("health_screen_enabled", val_bool(c.health_screen_enabled)),
    ])
}

fn export_canned_message(c: &protobufs::module_config::CannedMessageConfig) -> Value {
    map_from(vec![
        ("rotary1_enabled", val_bool(c.rotary1_enabled)),
        ("inputbroker_pin_a", val_u32(c.inputbroker_pin_a)),
        ("inputbroker_pin_b", val_u32(c.inputbroker_pin_b)),
        ("inputbroker_pin_press", val_u32(c.inputbroker_pin_press)),
        ("inputbroker_event_cw", val_i32(c.inputbroker_event_cw)),
        ("inputbroker_event_ccw", val_i32(c.inputbroker_event_ccw)),
        (
            "inputbroker_event_press",
            val_i32(c.inputbroker_event_press),
        ),
        ("updown1_enabled", val_bool(c.updown1_enabled)),
        ("send_bell", val_bool(c.send_bell)),
    ])
}

fn export_audio(c: &protobufs::module_config::AudioConfig) -> Value {
    map_from(vec![
        ("codec2_enabled", val_bool(c.codec2_enabled)),
        ("ptt_pin", val_u32(c.ptt_pin)),
        ("bitrate", val_i32(c.bitrate)),
        ("i2s_ws", val_u32(c.i2s_ws)),
        ("i2s_sd", val_u32(c.i2s_sd)),
        ("i2s_din", val_u32(c.i2s_din)),
        ("i2s_sck", val_u32(c.i2s_sck)),
    ])
}

fn export_remote_hardware(c: &protobufs::module_config::RemoteHardwareConfig) -> Value {
    map_from(vec![
        ("enabled", val_bool(c.enabled)),
        (
            "allow_undefined_pin_access",
            val_bool(c.allow_undefined_pin_access),
        ),
    ])
}

fn export_neighbor_info(c: &protobufs::module_config::NeighborInfoConfig) -> Value {
    map_from(vec![
        ("enabled", val_bool(c.enabled)),
        ("update_interval", val_u32(c.update_interval)),
        ("transmit_over_lora", val_bool(c.transmit_over_lora)),
    ])
}

fn export_ambient_lighting(c: &protobufs::module_config::AmbientLightingConfig) -> Value {
    map_from(vec![
        ("led_state", val_bool(c.led_state)),
        ("current", val_u32(c.current)),
        ("red", val_u32(c.red)),
        ("green", val_u32(c.green)),
        ("blue", val_u32(c.blue)),
    ])
}

fn export_detection_sensor(c: &protobufs::module_config::DetectionSensorConfig) -> Value {
    map_from(vec![
        ("enabled", val_bool(c.enabled)),
        ("minimum_broadcast_secs", val_u32(c.minimum_broadcast_secs)),
        ("state_broadcast_secs", val_u32(c.state_broadcast_secs)),
        ("send_bell", val_bool(c.send_bell)),
        ("name", val_str(&c.name)),
        ("monitor_pin", val_u32(c.monitor_pin)),
        ("detection_trigger_type", val_i32(c.detection_trigger_type)),
        ("use_pullup", val_bool(c.use_pullup)),
    ])
}

fn export_paxcounter(c: &protobufs::module_config::PaxcounterConfig) -> Value {
    map_from(vec![
        ("enabled", val_bool(c.enabled)),
        (
            "paxcounter_update_interval",
            val_u32(c.paxcounter_update_interval),
        ),
        ("wifi_threshold", val_i32(c.wifi_threshold)),
        ("ble_threshold", val_i32(c.ble_threshold)),
    ])
}

fn export_channel(ch: &protobufs::Channel) -> Value {
    let role_str = match channel::Role::try_from(ch.role) {
        Ok(r) => r.as_str_name().to_string(),
        Err(_) => ch.role.to_string(),
    };

    let settings = ch.settings.as_ref();
    let name = settings.map(|s| s.name.clone()).unwrap_or_default();
    let psk = settings.map(|s| hex_encode(&s.psk)).unwrap_or_default();
    let uplink = settings.is_some_and(|s| s.uplink_enabled);
    let downlink = settings.is_some_and(|s| s.downlink_enabled);
    let position_precision = settings
        .and_then(|s| s.module_settings.as_ref())
        .map(|m| m.position_precision)
        .unwrap_or(0);

    map_from(vec![
        ("index", val_i32(ch.index)),
        ("role", val_str(&role_str)),
        ("name", val_str(&name)),
        ("psk", val_str(&psk)),
        ("uplink_enabled", val_bool(uplink)),
        ("downlink_enabled", val_bool(downlink)),
        ("position_precision", val_u32(position_precision)),
    ])
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

// ── Import helpers ─────────────────────────────────────────────────

fn yaml_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        _ => format!("{:?}", v),
    }
}

fn import_config_section(
    section: &str,
    fields: &serde_yaml::Mapping,
    config: &protobufs::LocalConfig,
) -> anyhow::Result<protobufs::config::PayloadVariant> {
    // We reuse the config set infrastructure by converting YAML fields to string values
    // and applying them one by one to a cloned config section
    use super::config::apply_config_field;

    let mut result: Option<protobufs::config::PayloadVariant> = None;

    for (key, value) in fields {
        let field_name = key
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Field name must be a string"))?;
        let val_str = yaml_to_string(value);
        result = Some(apply_config_field(section, field_name, &val_str, config)?);
    }

    result.ok_or_else(|| anyhow::anyhow!("Section '{}' has no fields", section))
}

fn import_module_config_section(
    section: &str,
    fields: &serde_yaml::Mapping,
    module: &protobufs::LocalModuleConfig,
) -> anyhow::Result<protobufs::module_config::PayloadVariant> {
    use super::config::apply_module_config_field;

    let mut result: Option<protobufs::module_config::PayloadVariant> = None;

    for (key, value) in fields {
        let field_name = key
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Field name must be a string"))?;
        let val_str = yaml_to_string(value);
        result = Some(apply_module_config_field(
            section, field_name, &val_str, module,
        )?);
    }

    result.ok_or_else(|| anyhow::anyhow!("Section '{}' has no fields", section))
}

fn import_channels(value: &Value) -> anyhow::Result<Vec<protobufs::Channel>> {
    let Value::Sequence(list) = value else {
        bail!("'channels' must be a list");
    };

    let mut channels = Vec::new();

    for item in list {
        let Value::Mapping(m) = item else {
            bail!("Each channel must be a mapping");
        };

        let index = m
            .get(Value::String("index".to_string()))
            .and_then(|v| v.as_u64())
            .ok_or_else(|| anyhow::anyhow!("Channel missing 'index' field"))?
            as i32;

        let role_str = m
            .get(Value::String("role".to_string()))
            .and_then(|v| v.as_str())
            .unwrap_or("SECONDARY");

        let role = match role_str.to_uppercase().as_str() {
            "PRIMARY" => channel::Role::Primary as i32,
            "SECONDARY" => channel::Role::Secondary as i32,
            "DISABLED" => channel::Role::Disabled as i32,
            _ => channel::Role::Secondary as i32,
        };

        let name = m
            .get(Value::String("name".to_string()))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let psk_hex = m
            .get(Value::String("psk".to_string()))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let psk = if psk_hex.is_empty() {
            vec![1] // default
        } else {
            hex_decode(psk_hex)?
        };

        let uplink = m
            .get(Value::String("uplink_enabled".to_string()))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let downlink = m
            .get(Value::String("downlink_enabled".to_string()))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let position_precision = m
            .get(Value::String("position_precision".to_string()))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        channels.push(protobufs::Channel {
            index,
            role,
            settings: Some(ChannelSettings {
                name,
                psk,
                uplink_enabled: uplink,
                downlink_enabled: downlink,
                module_settings: Some(protobufs::ModuleSettings {
                    position_precision,
                    is_client_muted: false,
                }),
                ..Default::default()
            }),
        });
    }

    Ok(channels)
}

fn hex_decode(hex: &str) -> anyhow::Result<Vec<u8>> {
    if hex.is_empty() {
        return Ok(vec![]);
    }
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
