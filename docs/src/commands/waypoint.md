# Waypoints: waypoint

Send, delete, and list waypoints on the mesh network. Waypoints are named geographic points that appear on the map in compatible Meshtastic clients.

## `waypoint send`

Broadcast or unicast a new waypoint to the mesh.

```bash
# Send a waypoint broadcast to all nodes
meshtastic-cli waypoint send --name "Base Camp" --lat 40.4168 --lon -3.7038

# Send a waypoint with full options
meshtastic-cli waypoint send \
  --name "Checkpoint A" \
  --lat 40.4168 \
  --lon -3.7038 \
  --alt 650 \
  --icon 9410 \
  --expire 3600 \
  --dest 04e1c43b
```

| Option | Description |
|---|---|
| `--name` | Waypoint name, up to 30 characters (required) |
| `--lat` | Latitude in decimal degrees (required) |
| `--lon` | Longitude in decimal degrees (required) |
| `--alt` | Altitude in meters (optional) |
| `--icon` | Unicode code point for the waypoint icon displayed in clients (optional) |
| `--expire` | Seconds until the waypoint expires (optional, omit for no expiry) |
| `--dest` | Target node ID in hex for a unicast waypoint (omit to broadcast) |
| `--to` | Target node name for a unicast waypoint (omit to broadcast) |

---

## `waypoint delete`

Delete a waypoint by its numeric ID.

```bash
meshtastic-cli waypoint delete --id 42
```

| Option | Description |
|---|---|
| `--id` | Numeric waypoint ID to delete (required) |

---

## `waypoint list`

List all waypoints known to the local node. Listens for incoming waypoint packets with a configurable timeout.

```bash
meshtastic-cli waypoint list
```
