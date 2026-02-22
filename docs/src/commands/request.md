# Remote Requests: request

Request data from remote nodes.

## `request telemetry`

Request telemetry data from a remote node. Use `--type` to select a specific telemetry variant (default: device).

```bash
# Request device telemetry (battery, voltage, channel utilization)
mttctl request telemetry --dest 04e1c43b

# Request environment telemetry (temperature, humidity, pressure)
mttctl request telemetry --to Pedro --type environment

# Request air quality metrics (PM1.0, PM2.5, PM10.0, CO2, VOC)
mttctl request telemetry --dest 04e1c43b --type air-quality

# Request power metrics (voltage/current per channel)
mttctl request telemetry --dest 04e1c43b --type power

# Request local stats (uptime, packets tx/rx, air utilization)
mttctl request telemetry --dest 04e1c43b --type local-stats

# Request health metrics (heart rate, SpO2)
mttctl request telemetry --dest 04e1c43b --type health

# Request host metrics (free memory, disk, load average)
mttctl request telemetry --dest 04e1c43b --type host
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |
| `--type` | Telemetry type: `device`, `environment`, `air-quality`, `power`, `local-stats`, `health`, `host` (default: `device`) |
| `--timeout` | Timeout in seconds (default: 30) |

---

## `request position`

Request position data from a remote node.

```bash
# Request by node ID
mttctl request position --dest 04e1c43b
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |

---

## `request metadata`

Request device metadata (firmware version, hardware model, capabilities) from a remote node.

```bash
# Request by node ID
mttctl request metadata --dest 04e1c43b

# Request by name with custom timeout
mttctl request metadata --to Pedro --timeout 60
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |
| `--timeout` | Seconds to wait for response (default: 30) |

Example output:

```
Device metadata from Pedro (!04e1c43b):
  Firmware:     2.5.6.abc1234
  Hardware:     HELTEC_V3
  Device ID:    04e1c43b
  Capabilities: HasWifi, HasBluetooth
```
