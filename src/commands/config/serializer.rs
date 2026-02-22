use meshtastic::protobufs;
use serde_json::json;

// ── JSON helpers ───────────────────────────────────────────────────

pub(super) fn fmt_enum<T: std::fmt::Debug>(value: i32, f: impl FnOnce(i32) -> Option<T>) -> String {
    f(value)
        .map(|v| format!("{:?}", v))
        .unwrap_or_else(|| value.to_string())
}

pub(super) fn config_device_json(c: &protobufs::config::DeviceConfig) -> serde_json::Value {
    json!({
        "role": fmt_enum(c.role, |v| protobufs::config::device_config::Role::try_from(v).ok()),
        "node_info_broadcast_secs": c.node_info_broadcast_secs,
        "double_tap_as_button_press": c.double_tap_as_button_press,
        "disable_triple_click": c.disable_triple_click,
        "led_heartbeat_disabled": c.led_heartbeat_disabled,
        "rebroadcast_mode": fmt_enum(c.rebroadcast_mode, |v| protobufs::config::device_config::RebroadcastMode::try_from(v).ok()),
    })
}

pub(super) fn config_position_json(c: &protobufs::config::PositionConfig) -> serde_json::Value {
    json!({
        "position_broadcast_secs": c.position_broadcast_secs,
        "position_broadcast_smart_enabled": c.position_broadcast_smart_enabled,
        "fixed_position": c.fixed_position,
        "gps_update_interval": c.gps_update_interval,
        "position_flags": c.position_flags,
        "gps_mode": fmt_enum(c.gps_mode, |v| protobufs::config::position_config::GpsMode::try_from(v).ok()),
    })
}

pub(super) fn config_power_json(c: &protobufs::config::PowerConfig) -> serde_json::Value {
    json!({
        "is_power_saving": c.is_power_saving,
        "on_battery_shutdown_after_secs": c.on_battery_shutdown_after_secs,
        "adc_multiplier_override": c.adc_multiplier_override,
        "wait_bluetooth_secs": c.wait_bluetooth_secs,
        "sds_secs": c.sds_secs,
        "ls_secs": c.ls_secs,
        "min_wake_secs": c.min_wake_secs,
    })
}

pub(super) fn config_network_json(c: &protobufs::config::NetworkConfig) -> serde_json::Value {
    json!({
        "wifi_enabled": c.wifi_enabled,
        "wifi_ssid": c.wifi_ssid,
        "ntp_server": c.ntp_server,
        "eth_enabled": c.eth_enabled,
        "address_mode": fmt_enum(c.address_mode, |v| protobufs::config::network_config::AddressMode::try_from(v).ok()),
        "ipv6_enabled": c.ipv6_enabled,
    })
}

pub(super) fn config_display_json(c: &protobufs::config::DisplayConfig) -> serde_json::Value {
    json!({
        "screen_on_secs": c.screen_on_secs,
        "auto_screen_carousel_secs": c.auto_screen_carousel_secs,
        "flip_screen": c.flip_screen,
        "units": fmt_enum(c.units, |v| protobufs::config::display_config::DisplayUnits::try_from(v).ok()),
        "oled": fmt_enum(c.oled, |v| protobufs::config::display_config::OledType::try_from(v).ok()),
        "heading_bold": c.heading_bold,
        "use_12h_clock": c.use_12h_clock,
    })
}

pub(super) fn config_lora_json(c: &protobufs::config::LoRaConfig) -> serde_json::Value {
    json!({
        "region": fmt_enum(c.region, |v| protobufs::config::lo_ra_config::RegionCode::try_from(v).ok()),
        "modem_preset": fmt_enum(c.modem_preset, |v| protobufs::config::lo_ra_config::ModemPreset::try_from(v).ok()),
        "use_preset": c.use_preset,
        "bandwidth": c.bandwidth,
        "spread_factor": c.spread_factor,
        "coding_rate": c.coding_rate,
        "frequency_offset": c.frequency_offset,
        "hop_limit": c.hop_limit,
        "tx_enabled": c.tx_enabled,
        "tx_power": c.tx_power,
        "channel_num": c.channel_num,
        "override_duty_cycle": c.override_duty_cycle,
        "override_frequency": c.override_frequency,
        "ignore_mqtt": c.ignore_mqtt,
        "config_ok_to_mqtt": c.config_ok_to_mqtt,
    })
}

pub(super) fn config_bluetooth_json(c: &protobufs::config::BluetoothConfig) -> serde_json::Value {
    json!({
        "enabled": c.enabled,
        "mode": fmt_enum(c.mode, |v| protobufs::config::bluetooth_config::PairingMode::try_from(v).ok()),
        "fixed_pin": c.fixed_pin,
    })
}

pub(super) fn config_security_json(c: &protobufs::config::SecurityConfig) -> serde_json::Value {
    json!({
        "is_managed": c.is_managed,
        "serial_enabled": c.serial_enabled,
        "debug_log_api_enabled": c.debug_log_api_enabled,
        "admin_channel_enabled": c.admin_channel_enabled,
        "public_key_len": c.public_key.len(),
        "private_key_len": c.private_key.len(),
        "admin_key_count": c.admin_key.len(),
    })
}

pub(super) fn config_mqtt_json(c: &protobufs::module_config::MqttConfig) -> serde_json::Value {
    json!({
        "enabled": c.enabled,
        "address": c.address,
        "username": c.username,
        "encryption_enabled": c.encryption_enabled,
        "json_enabled": c.json_enabled,
        "tls_enabled": c.tls_enabled,
        "root": c.root,
        "proxy_to_client_enabled": c.proxy_to_client_enabled,
        "map_reporting_enabled": c.map_reporting_enabled,
    })
}

pub(super) fn config_serial_json(c: &protobufs::module_config::SerialConfig) -> serde_json::Value {
    json!({
        "enabled": c.enabled,
        "echo": c.echo,
        "rxd": c.rxd,
        "txd": c.txd,
        "baud": fmt_enum(c.baud, |v| protobufs::module_config::serial_config::SerialBaud::try_from(v).ok()),
        "timeout": c.timeout,
        "mode": fmt_enum(c.mode, |v| protobufs::module_config::serial_config::SerialMode::try_from(v).ok()),
    })
}

pub(super) fn config_telemetry_json(
    c: &protobufs::module_config::TelemetryConfig,
) -> serde_json::Value {
    json!({
        "device_update_interval": c.device_update_interval,
        "environment_update_interval": c.environment_update_interval,
        "environment_measurement_enabled": c.environment_measurement_enabled,
        "environment_screen_enabled": c.environment_screen_enabled,
        "air_quality_enabled": c.air_quality_enabled,
        "air_quality_interval": c.air_quality_interval,
        "power_measurement_enabled": c.power_measurement_enabled,
        "power_update_interval": c.power_update_interval,
    })
}

pub(super) fn config_neighbor_info_json(
    c: &protobufs::module_config::NeighborInfoConfig,
) -> serde_json::Value {
    json!({
        "enabled": c.enabled,
        "update_interval": c.update_interval,
        "transmit_over_lora": c.transmit_over_lora,
    })
}
