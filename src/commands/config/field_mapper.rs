use anyhow::bail;
use meshtastic::protobufs;

use crate::commands::parsers::{
    parse_bool, parse_enum_i32, parse_f32, parse_i32, parse_u32, parse_u64,
};

pub fn apply_config_field(
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

pub fn apply_module_config_field(
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
