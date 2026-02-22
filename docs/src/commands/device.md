# Device Management: device

Device management commands: reboot, reboot-ota, enter-dfu, shutdown, factory reset variants, reset-nodedb, set-time, canned messages, and ringtone. Reboot and shutdown support targeting the local device (default) or a remote node.

## `device reboot`

Reboot the connected device or a remote node.

```bash
# Reboot local device (5 second delay)
mttctl device reboot

# Reboot with custom delay
mttctl device reboot --delay 10

# Reboot a remote node by ID
mttctl device reboot --dest 04e1c43b

# Reboot a remote node by name
mttctl device reboot --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex. Omit to target local device |
| `--to` | Target node name. Omit to target local device |
| `--delay` | Seconds before rebooting (default: 5) |

---

## `device reboot-ota`

Reboot the device into OTA (Over-The-Air) firmware update mode. This is specific to ESP32-based Meshtastic hardware. Supports targeting the local device or a remote node.

```bash
# Reboot local device into OTA mode
mttctl device reboot-ota

# Reboot remote node into OTA mode
mttctl device reboot-ota --dest 04e1c43b
mttctl device reboot-ota --to Pedro

# Custom delay
mttctl device reboot-ota --delay 10
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex. Omit to target local device |
| `--to` | Target node name. Omit to target local device |
| `--delay` | Seconds before rebooting into OTA mode (default: 5) |

---

## `device enter-dfu`

Enter Device Firmware Upgrade (DFU) mode. This is specific to NRF52-based Meshtastic hardware (e.g., RAK devices). The device will appear as a USB mass storage device after entering DFU mode, allowing firmware file drops.

```bash
mttctl device enter-dfu
```

---

## `device shutdown`

Shut down the connected device or a remote node.

```bash
# Shutdown local device
mttctl device shutdown

# Shutdown with custom delay
mttctl device shutdown --delay 10

# Shutdown a remote node
mttctl device shutdown --dest 04e1c43b
```

| Option | Description |
|---|---|
| `--dest` | Target node ID in hex. Omit to target local device |
| `--to` | Target node name. Omit to target local device |
| `--delay` | Seconds before shutting down (default: 5) |

---

## `device factory-reset`

Restore the device to factory defaults. This erases all configuration and stored data but **preserves BLE bonds**.

```bash
mttctl device factory-reset
```

---

## `device factory-reset-device`

Perform a full factory reset that also **wipes all BLE bonds**. Use this when you want to completely reset the device as if it were brand new, including removing all previously paired Bluetooth devices.

```bash
mttctl device factory-reset-device
```

---

## `device reset-nodedb`

Clear the device's entire node database. This removes all known nodes from the local NodeDB.

```bash
mttctl device reset-nodedb
```

---

## `device set-time`

Set the device clock. Uses the current system time if no timestamp is provided.

```bash
# Set time from system clock
mttctl device set-time

# Set time from a specific Unix timestamp
mttctl device set-time 1708444800
```

| Option | Description |
|---|---|
| `[TIMESTAMP]` | Unix timestamp in seconds. Uses system time if omitted |

---

## `device set-canned-message`

Set the canned messages stored on the device. Messages are separated by `|` and can be selected quickly from a compatible Meshtastic client.

```bash
mttctl device set-canned-message "Yes|No|Help|On my way|Call me"
```

| Option | Description |
|---|---|
| `<MESSAGES>` | Pipe-separated list of canned messages (required) |

---

## `device get-canned-message`

Display the canned messages currently configured on the device. Requests the canned message module config from the device and waits for the response.

```bash
mttctl device get-canned-message

# Custom timeout
mttctl device get-canned-message --timeout 60
```

| Option | Description |
|---|---|
| `--timeout` | Seconds to wait for the device response (default: 30) |

Example output:

```
Canned messages:
  1: Yes
  2: No
  3: Help
  4: On my way
  5: Call me
```

---

## `device set-ringtone`

Set the notification ringtone on the device. The ringtone is provided in RTTTL (Ring Tone Text Transfer Language) format.

```bash
mttctl device set-ringtone "scale:d=4,o=5,b=120:c,e,g,c6"
```

| Option | Description |
|---|---|
| `<RINGTONE>` | Ringtone string in RTTTL format (required) |

---

## `device get-ringtone`

Display the notification ringtone currently stored on the device.

```bash
mttctl device get-ringtone

# Custom timeout
mttctl device get-ringtone --timeout 60
```

| Option | Description |
|---|---|
| `--timeout` | Seconds to wait for the device response (default: 30) |

Example output:

```
Ringtone: scale:d=4,o=5,b=120:c,e,g,c6
```
