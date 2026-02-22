# Channels: channel

Manage device channels: list, add, delete, modify properties, and generate a QR code for sharing.

## `channel list`

List all configured channels with their role, encryption, and uplink/downlink status.

```bash
meshtastic-cli channel list
```

Example output:

```
Channels
  [0]    Default        Primary      Default key  uplink: false downlink: false
  [1]    Team           Secondary    AES-256      uplink: false downlink: false
```

---

## `channel add`

Add a new secondary channel. The channel is placed in the first available slot (indices 1-7).

```bash
# Add with default encryption key
meshtastic-cli channel add "Team"

# Add with a random AES-256 key
meshtastic-cli channel add "Secure" --psk random

# Add with no encryption
meshtastic-cli channel add "Open" --psk none

# Add with a specific AES-128 key (32 hex characters)
meshtastic-cli channel add "Custom" --psk d4f1bb3a2029075960bcffabcf4e6901
```

| Option | Description |
|---|---|
| `<NAME>` | Channel name, up to 11 characters (required) |
| `--psk` | Pre-shared key: `none`, `default`, `random`, or hex-encoded key (default: `default`) |

---

## `channel del`

Delete a channel by index. Cannot delete the primary channel (index 0).

```bash
meshtastic-cli channel del 1
```

---

## `channel set`

Set a property on a specific channel.

```bash
# Rename a channel
meshtastic-cli channel set 1 name "NewName"

# Change encryption key
meshtastic-cli channel set 1 psk random

# Enable MQTT uplink
meshtastic-cli channel set 1 uplink_enabled true

# Enable MQTT downlink
meshtastic-cli channel set 1 downlink_enabled true

# Set position precision
meshtastic-cli channel set 0 position_precision 14
```

| Field | Description |
|---|---|
| `name` | Channel name (up to 11 characters) |
| `psk` | Pre-shared key (`none`, `default`, `random`, or hex) |
| `uplink_enabled` | Forward mesh messages to MQTT |
| `downlink_enabled` | Forward MQTT messages to mesh |
| `position_precision` | Bits of precision for position data |

---

## `channel qr`

Generate a QR code and shareable meshtastic:// URL for the current channel configuration. By default the QR code is printed directly to the terminal using Unicode block characters. Use `--output` to save as a PNG or SVG image file. Use `--all` to generate a separate QR code for each active channel individually.

```bash
# Print combined QR code to terminal (all active channels)
meshtastic-cli channel qr

# Export combined QR as PNG image (512x512 minimum)
meshtastic-cli channel qr --output channels.png

# Export combined QR as SVG image
meshtastic-cli channel qr --output channels.svg

# Print individual QR code per active channel to terminal
meshtastic-cli channel qr --all
```

| Option | Description |
|---|---|
| `--output` | File path for image export. Supports `.png` and `.svg` formats. Prints to terminal if omitted. Cannot be combined with `--all`. |
| `--all` | Generate one QR code per active channel, printed to terminal. Cannot be combined with `--output`. |

Example output (terminal):

```
[block character QR code rendered in terminal]

URL: https://meshtastic.org/e/#ENCODED...
```

Example output (file export):

```
ok QR code saved to channels.png

URL: https://meshtastic.org/e/#ENCODED...
```

Example output (`--all`, two active channels):

```
Channel 0: Default
[block character QR code]
URL: https://meshtastic.org/e/#ENCODED_CH0...

Channel 1: Team
[block character QR code]
URL: https://meshtastic.org/e/#ENCODED_CH1...
```
