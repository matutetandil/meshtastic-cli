# Watch: watch

Displays the node table as a live-updating view that refreshes periodically in place, similar to the `watch` Unix utility applied to the `nodes` output. Press Ctrl+C to stop.

```bash
# Watch node table, refresh every 30 seconds (default)
mttctl watch

# Refresh every 10 seconds
mttctl watch --interval 10

# Watch with a custom field set
mttctl watch --fields id,name,battery,snr --interval 15
```

| Option | Description |
|---|---|
| `--interval` | Refresh interval in seconds (default: 30) |
| `--fields` | Comma-separated list of columns to display (same values as `nodes --fields`) |

Example output (refreshes in place):

```
mttctl watch  --  refreshing every 30s  --  last update: 15:30:00  --  Ctrl+C to stop

  ID          Name          Battery   SNR     Hops   Last Heard
  !04e1c43b   Pedro         85%       8.5     0      just now
  !a1b2c3d4   Maria         72%       6.0     1      2m ago
  !e5f6a7b8   Relay-1       --        4.5     2      5m ago
```
