# Position: position

GPS position commands: get, set, and remove.

## `position get`

Display the current GPS position of the local node.

```bash
meshtastic-cli position get
```

---

## `position set`

Set a fixed GPS position on the device. Requires latitude and longitude; altitude and broadcast flags are optional. Once a fixed position is set, the device broadcasts this position instead of using live GPS data.

```bash
# Set position with latitude and longitude
meshtastic-cli position set 40.4168 -3.7038

# Set position with altitude (in meters)
meshtastic-cli position set 40.4168 -3.7038 650

# Set position with named broadcast flags
meshtastic-cli position set 40.4168 -3.7038 650 --flags "ALTITUDE,TIMESTAMP,SPEED"

# Set position with numeric bitmask (equivalent to above: 1 + 128 + 512 = 641)
meshtastic-cli position set 40.4168 -3.7038 650 --flags 641

# Set position with hex bitmask
meshtastic-cli position set 40.4168 -3.7038 650 --flags 0x281
```

| Option | Description |
|---|---|
| `<LATITUDE>` | Latitude in decimal degrees (required) |
| `<LONGITUDE>` | Longitude in decimal degrees (required) |
| `<ALTITUDE>` | Altitude in meters (optional) |
| `--flags` | Position broadcast field flags (optional). Accepts comma-separated names (`ALTITUDE`, `ALTITUDE_MSL`, `GEOIDAL_SEPARATION`, `DOP`, `HVDOP`, `SATINVIEW`, `SEQ_NO`, `TIMESTAMP`, `HEADING`, `SPEED`) or a numeric bitmask (decimal or `0x` hex). |

---

## `position remove`

Remove the fixed GPS position from the device. After removal, the device will return to using live GPS data if a GPS module is available.

```bash
meshtastic-cli position remove
```
