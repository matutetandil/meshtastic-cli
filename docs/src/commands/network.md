# Network: ping, traceroute

## `ping`

Sends a ping to a specific node and measures the round-trip time by waiting for an ACK.

```bash
# Ping by node ID
meshtastic-cli ping --dest 04e1c43b

# Ping by name
meshtastic-cli ping --to Pedro

# Custom timeout (default: 30s)
meshtastic-cli ping --dest 04e1c43b --timeout 60
```

| Option      | Description                                            |
|-------------|--------------------------------------------------------|
| `--dest`    | Destination node ID in hex, `!` prefix optional (required unless `--to` is used) |
| `--to`      | Destination node name (required unless `--dest` is used) |
| `--timeout` | Seconds to wait for ACK (default: 30)                  |

Example output:

```
-> Pinging !04e1c43b (Pedro) (packet id: a1b2c3d4)...
ok ACK from !04e1c43b (Pedro) in 2.3s
```

If the node doesn't respond:

```
-> Pinging !04e1c43b (Pedro) (packet id: a1b2c3d4)...
x Timeout after 30s -- no ACK from !04e1c43b (Pedro)
```

---

## `traceroute`

Traces the route to a destination node, showing each hop along the path with SNR (signal-to-noise ratio) values.

```bash
# Traceroute by node ID
meshtastic-cli traceroute --dest 04e1c43b

# Traceroute by name
meshtastic-cli traceroute --to Pedro

# Custom timeout (default: 60s)
meshtastic-cli traceroute --dest 04e1c43b --timeout 120
```

| Option | Description |
|---|---|
| `--dest` | Destination node ID in hex, `!` prefix optional (required unless `--to` is used) |
| `--to` | Destination node name (required unless `--dest` is used) |
| `--timeout` | Seconds to wait for response (default: 60) |

Example output:

```
-> Tracing route to Pedro (!04e1c43b)...

  1 !a1b2c3d4 (Local)
  2 !e5f6a7b8 (Relay-1)     SNR: 6.0 dB
  3 !04e1c43b (Pedro)        SNR: 8.5 dB

ok Route to Pedro (!04e1c43b) completed in 4.2s (2 hops)
```

If a return path differs from the forward path, both are shown separately.
