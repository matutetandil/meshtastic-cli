# Node Management: node

## `node set-owner`

Set the device owner name (long name and short name). The short name is auto-generated from the long name if omitted.

```bash
# Set long name (short name auto-generated as "PD")
meshtastic-cli node set-owner "Pedro"

# Set both long and short name
meshtastic-cli node set-owner "Pedro's Node" --short PN

# Multi-word names generate initials (e.g. "My Cool Node" -> "MCN")
meshtastic-cli node set-owner "My Cool Node"
```

| Option | Description |
|---|---|
| `<NAME>` | Long name for the device, up to 40 characters (required) |
| `--short` | Short name, up to 5 characters. Auto-generated if omitted |

---

## `node remove`

Remove a specific node from the local NodeDB. The node can be specified by hex ID or by name.

```bash
# Remove by node ID
meshtastic-cli node remove --dest 04e1c43b

# Remove by name
meshtastic-cli node remove --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex to remove (required unless `--to` is used) |
| `--to` | Node name to remove (required unless `--dest` is used) |

---

## `node set-favorite`

Mark a node as a favorite. Favorites are stored on the device and can be used for filtering in compatible clients.

```bash
# Mark by node ID
meshtastic-cli node set-favorite --dest 04e1c43b

# Mark by name
meshtastic-cli node set-favorite --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

---

## `node remove-favorite`

Remove a node from the favorites list.

```bash
meshtastic-cli node remove-favorite --dest 04e1c43b
meshtastic-cli node remove-favorite --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

---

## `node set-ignored`

Mark a node as ignored. Ignored nodes are filtered out of mesh activity on the local device.

```bash
meshtastic-cli node set-ignored --dest 04e1c43b
meshtastic-cli node set-ignored --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

---

## `node remove-ignored`

Remove a node from the ignored list.

```bash
meshtastic-cli node remove-ignored --dest 04e1c43b
meshtastic-cli node remove-ignored --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

---

## `node set-unmessageable`

Mark the local node as unmessageable (prevents others from sending direct messages to it) or restore it as messageable.

```bash
# Mark as unmessageable (default)
meshtastic-cli node set-unmessageable

# Explicitly mark as unmessageable
meshtastic-cli node set-unmessageable true

# Restore as messageable
meshtastic-cli node set-unmessageable false
```

| Option | Description |
|---|---|
| `[VALUE]` | `true` to mark as unmessageable, `false` to mark as messageable (default: `true`) |
