use colored::Colorize;
use meshtastic::protobufs;

// ── Print helpers ──────────────────────────────────────────────────

pub(super) fn section_header(name: &str) {
    println!("{}", name.bold().underline());
}

pub(super) fn field(label: &str, value: &str) {
    println!("  {:<40} {}", format!("{}:", label).dimmed(), value);
}

pub(super) fn field_enum<T: std::fmt::Debug>(
    label: &str,
    value: i32,
    f: impl FnOnce(i32) -> Option<T>,
) {
    let display = f(value)
        .map(|v| format!("{:?}", v))
        .unwrap_or_else(|| value.to_string());
    field(label, &display);
}

pub(super) fn not_received(section_name: &str) {
    section_header(section_name);
    println!("  {}", "(not received from device)".dimmed());
    println!();
}

// ── Config section printers ────────────────────────────────────────

pub(super) fn print_device(cfg: Option<&protobufs::config::DeviceConfig>) {
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

pub(super) fn print_position(cfg: Option<&protobufs::config::PositionConfig>) {
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

pub(super) fn print_power(cfg: Option<&protobufs::config::PowerConfig>) {
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

pub(super) fn print_network(cfg: Option<&protobufs::config::NetworkConfig>) {
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

pub(super) fn print_display(cfg: Option<&protobufs::config::DisplayConfig>) {
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

pub(super) fn print_lora(cfg: Option<&protobufs::config::LoRaConfig>) {
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

pub(super) fn print_bluetooth(cfg: Option<&protobufs::config::BluetoothConfig>) {
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

pub(super) fn print_security(cfg: Option<&protobufs::config::SecurityConfig>) {
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

pub(super) fn print_mqtt(cfg: Option<&protobufs::module_config::MqttConfig>) {
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

pub(super) fn print_serial(cfg: Option<&protobufs::module_config::SerialConfig>) {
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

pub(super) fn print_external_notification(
    cfg: Option<&protobufs::module_config::ExternalNotificationConfig>,
) {
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

pub(super) fn print_store_forward(cfg: Option<&protobufs::module_config::StoreForwardConfig>) {
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

pub(super) fn print_range_test(cfg: Option<&protobufs::module_config::RangeTestConfig>) {
    let Some(c) = cfg else {
        return not_received("Range Test");
    };
    section_header("Range Test");
    field("enabled", &c.enabled.to_string());
    field("sender", &c.sender.to_string());
    field("save", &c.save.to_string());
    println!();
}

pub(super) fn print_telemetry(cfg: Option<&protobufs::module_config::TelemetryConfig>) {
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

pub(super) fn print_canned_message(cfg: Option<&protobufs::module_config::CannedMessageConfig>) {
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

pub(super) fn print_audio(cfg: Option<&protobufs::module_config::AudioConfig>) {
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

pub(super) fn print_remote_hardware(cfg: Option<&protobufs::module_config::RemoteHardwareConfig>) {
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

pub(super) fn print_neighbor_info(cfg: Option<&protobufs::module_config::NeighborInfoConfig>) {
    let Some(c) = cfg else {
        return not_received("Neighbor Info");
    };
    section_header("Neighbor Info");
    field("enabled", &c.enabled.to_string());
    field("update_interval", &c.update_interval.to_string());
    field("transmit_over_lora", &c.transmit_over_lora.to_string());
    println!();
}

pub(super) fn print_ambient_lighting(
    cfg: Option<&protobufs::module_config::AmbientLightingConfig>,
) {
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

pub(super) fn print_detection_sensor(
    cfg: Option<&protobufs::module_config::DetectionSensorConfig>,
) {
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

pub(super) fn print_paxcounter(cfg: Option<&protobufs::module_config::PaxcounterConfig>) {
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
