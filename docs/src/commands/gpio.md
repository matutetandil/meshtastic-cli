# GPIO: gpio

Remote GPIO pin operations on mesh nodes. Requires the target node to have the remote hardware module enabled. GPIO mask values can be provided in decimal or `0x` hex format.

## `gpio write`

Write a value to GPIO pins on a remote node. The mask specifies which pins to affect; the value specifies the state to write to those pins.

```bash
# Set GPIO pin 4 high on a remote node (mask and value in decimal)
mttctl gpio write --dest 04e1c43b --mask 16 --value 16

# Set GPIO pin 4 high (mask and value in hex)
mttctl gpio write --dest 04e1c43b --mask 0x10 --value 0x10

# Set pin 4 high and pin 5 low
mttctl gpio write --to Pedro --mask 0x30 --value 0x10
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |
| `--mask` | Bitmask of GPIO pins to write (decimal or 0x hex) |
| `--value` | Values to write to the masked pins (decimal or 0x hex) |

---

## `gpio read`

Read the current state of GPIO pins from a remote node.

```bash
# Read pins 4 and 5 from a remote node (mask in decimal)
mttctl gpio read --dest 04e1c43b --mask 48

# Read using hex mask
mttctl gpio read --to Pedro --mask 0x30

# Custom timeout
mttctl gpio read --dest 04e1c43b --mask 0x10 --timeout 60
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |
| `--mask` | Bitmask of GPIO pins to read (decimal or 0x hex) |
| `--timeout` | Seconds to wait for the response (default: 30) |

Example output:

```
GPIO state from Pedro (!04e1c43b):
  Mask:  0x00000030
  Value: 0x00000010  (pin 4: HIGH, pin 5: LOW)
```

---

## `gpio watch`

Watch for GPIO state changes on a remote node. Runs continuously until interrupted with Ctrl+C. Each state change is printed with a timestamp.

```bash
# Watch pins 4 and 5
mttctl gpio watch --dest 04e1c43b --mask 0x30

# Watch by node name
mttctl gpio watch --to Pedro --mask 0x10
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex (required unless `--to` is used) |
| `--to` | Target node name (required unless `--dest` is used) |
| `--mask` | Bitmask of GPIO pins to watch (decimal or 0x hex) |

Example output:

```
-> Watching GPIO on Pedro (!04e1c43b) [mask: 0x00000030]. Press Ctrl+C to stop.

[15:30:02] Value changed: 0x00000010  (pin 4: HIGH, pin 5: LOW)
[15:31:15] Value changed: 0x00000030  (pin 4: HIGH, pin 5: HIGH)
[15:32:40] Value changed: 0x00000000  (pin 4: LOW, pin 5: LOW)
```
