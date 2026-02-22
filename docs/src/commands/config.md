# Configuration: config

Read, write, export, and import device and module configuration. Supports all 8 device config sections and 13 module config sections.

## `config get`

Display current configuration. Optionally specify a section to show only that section.

```bash
# Show all configuration sections
mttctl config get

# Show a specific section
mttctl config get lora
mttctl config get mqtt
mttctl config get device
```

Available sections:

| Device Config | Module Config |
|--------------|---------------|
| `device` | `mqtt` |
| `position` | `serial` |
| `power` | `external-notification` |
| `network` | `store-forward` |
| `display` | `range-test` |
| `lora` | `telemetry` |
| `bluetooth` | `canned-message` |
| `security` | `audio` |
| | `remote-hardware` |
| | `neighbor-info` |
| | `ambient-lighting` |
| | `detection-sensor` |
| | `paxcounter` |

Example output:

```
LoRa
  region:                                  Us
  modem_preset:                            LongFast
  use_preset:                              true
  hop_limit:                               3
  tx_enabled:                              true
  tx_power:                                30
  ...
```

---

## `config set`

Set a configuration value. The key uses `section.field` format. The device will reboot after applying changes.

```bash
# Set LoRa region
mttctl config set lora.region Eu868

# Change device role
mttctl config set device.role Router

# Set hop limit
mttctl config set lora.hop_limit 5

# Enable MQTT
mttctl config set mqtt.enabled true

# Set WiFi credentials
mttctl config set network.wifi_ssid "MyNetwork"
mttctl config set network.wifi_psk "MyPassword"
```

For enum fields, use the human-readable name (case-insensitive). Run `config get <section>` to see current values and available field names.

Example output:

```
-> Setting lora.region = Eu868
! Device will reboot to apply changes.
ok Configuration updated.
```

---

## `config begin-edit`

Signal the device to begin collecting a batch of configuration changes. Use this before a sequence of `config set` calls to apply them all in a single transaction rather than rebooting after each change.

```bash
mttctl config begin-edit
mttctl config set lora.region Eu868
mttctl config set lora.hop_limit 5
mttctl config set device.role Router
mttctl config commit-edit
```

---

## `config commit-edit`

Signal the device to commit and apply all configuration changes queued since the last `config begin-edit`. The device will reboot once to apply all pending changes.

```bash
mttctl config commit-edit
```

---

## `config set-modem-preset`

Set the LoRa modem preset directly by name, without having to go through `config set lora.modem_preset`. Valid preset names are case-insensitive.

```bash
mttctl config set-modem-preset LongFast
mttctl config set-modem-preset ShortTurbo
mttctl config set-modem-preset MediumSlow
```

Available presets:

| Preset | Description |
|---|---|
| `LongFast` | Long range, faster throughput (default) |
| `LongSlow` | Long range, slower throughput |
| `VeryLongSlow` | Maximum range, very slow |
| `MediumSlow` | Medium range, slower |
| `MediumFast` | Medium range, faster |
| `ShortSlow` | Short range, slower |
| `ShortFast` | Short range, fastest throughput |
| `LongModerate` | Long range, moderate throughput |
| `ShortTurbo` | Short range, maximum throughput |

---

## `config ch-add-url`

Add channels from a meshtastic:// URL without replacing existing channels. This differs from `config set-url`, which replaces all current channels with those from the URL.

```bash
mttctl config ch-add-url "https://meshtastic.org/e/#ENCODED..."
```

| Option | Description |
|---|---|
| `<URL>` | Meshtastic configuration URL (required) |

---

## `config export`

Exports the full device configuration (device config, module config, and channels) as YAML. Useful for backups, sharing configurations, or migrating between devices.

```bash
# Print config to stdout
mttctl config export

# Save to a file
mttctl config export --file backup.yaml
```

| Option | Description |
|---|---|
| `--file` | Output file path. If omitted, prints YAML to stdout |

Example output (truncated):

```yaml
bluetooth:
  enabled: true
  fixed_pin: 123456
  mode: 1
device:
  role: 0
  node_info_broadcast_secs: 900
  ...
lora:
  region: 1
  modem_preset: 3
  hop_limit: 3
  ...
mqtt:
  enabled: false
  address: mqtt.meshtastic.org
  ...
channels:
  - index: 0
    role: PRIMARY
    name: ''
    psk: '01'
    uplink_enabled: false
    downlink_enabled: false
    position_precision: 0
  - index: 1
    role: SECONDARY
    name: Team
    psk: d4f1bb3a2029075960bcffabcf4e6901...
    ...
```

---

## `config import`

Imports and applies configuration from a YAML file. The file format matches the output of `config export`. Sections not present in the file are left unchanged. The device will reboot after applying config changes.

```bash
mttctl config import backup.yaml
```

| Option | Description |
|---|---|
| `<FILE>` | Path to the YAML configuration file (required) |

Example output:

```
-> Importing configuration from backup.yaml...
ok Imported 8 config sections, 13 module sections, 2 channels.
! Device will reboot to apply configuration changes.
```

---

## `config set-ham`

Configure the device for licensed Ham radio operation. Sets the callsign as the long name, enables long-range LoRa settings, and disables encryption as required by Ham regulations. Optionally set TX power and frequency.

```bash
# Set Ham mode with callsign
mttctl config set-ham KD2ABC

# Set Ham mode with custom TX power and frequency
mttctl config set-ham KD2ABC --tx-power 17 --frequency 906.875
```

| Option | Description |
|---|---|
| `<CALLSIGN>` | Ham radio callsign to set as device name (required) |
| `--tx-power` | Transmit power in dBm (optional) |
| `--frequency` | Frequency in MHz (optional) |

---

## `config set-url`

Apply channels and LoRa configuration from a meshtastic:// URL. These URLs are typically generated by the Meshtastic app or web client for sharing device configurations. **This replaces all existing channels** with those defined in the URL. To add channels without replacing existing ones, use `config ch-add-url`.

```bash
mttctl config set-url "https://meshtastic.org/e/#ENCODED..."
```

| Option | Description |
|---|---|
| `<URL>` | Meshtastic configuration URL (required) |
