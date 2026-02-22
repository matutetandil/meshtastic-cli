# MQTT Bridge: mqtt

Bidirectional MQTT bridge. Subscribes to incoming mesh packets and republishes them to an MQTT broker as JSON, and optionally subscribes to an MQTT topic to inject messages back into the mesh. Useful for integrating a Meshtastic mesh into home automation, dashboards, or data pipelines without enabling the built-in MQTT module on the device.

## `mqtt bridge`

```bash
# Bridge to a local MQTT broker with default topic prefix
mttctl mqtt bridge --broker mqtt://localhost:1883

# Bridge with authentication
mttctl mqtt bridge \
  --broker mqtt://broker.example.com:1883 \
  --username myuser \
  --password mypassword

# Bridge with a custom topic prefix (default: meshtastic)
mttctl mqtt bridge \
  --broker mqtt://localhost:1883 \
  --topic my-mesh

# Bridge without bidirectional injection (publish only)
mttctl mqtt bridge \
  --broker mqtt://localhost:1883 \
  --no-downlink
```

| Option | Description |
|---|---|
| `--broker` | MQTT broker URL including scheme and port, e.g. `mqtt://localhost:1883` (required) |
| `--username` | MQTT username for authenticated brokers (optional) |
| `--password` | MQTT password for authenticated brokers (optional) |
| `--topic` | Topic prefix for all published messages (default: `meshtastic`) |
| `--no-downlink` | Disable the downlink subscription (publish-only mode) |

## Topic Format

Published topics follow this pattern:

```
<prefix>/<node-id>/<port-name>
```

Example topics published by the bridge:

```
meshtastic/04e1c43b/text
meshtastic/04e1c43b/position
meshtastic/04e1c43b/telemetry/device
```

Each message is a JSON object containing the decoded packet fields plus metadata (timestamp, sender node ID, SNR, RSSI, hops).

## Downlink (MQTT to Mesh)

The downlink topic for injecting messages into the mesh:

```
<prefix>/downlink/send
```

Post a JSON payload to this topic to send a text message via the bridge:

```json
{ "text": "hello mesh", "channel": 0 }
```
