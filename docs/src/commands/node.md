# Node Management: node

## `node set-owner`

Set the device owner name (long name and short name). The short name is auto-generated from the long name if omitted.

```bash
# Set long name (short name auto-generated as "PD")
mttctl node set-owner "Pedro"

# Set both long and short name
mttctl node set-owner "Pedro's Node" --short PN

# Multi-word names generate initials (e.g. "My Cool Node" -> "MCN")
mttctl node set-owner "My Cool Node"
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
mttctl node remove --dest 04e1c43b

# Remove by name
mttctl node remove --to Pedro
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
mttctl node set-favorite --dest 04e1c43b

# Mark by name
mttctl node set-favorite --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

---

## `node remove-favorite`

Remove a node from the favorites list.

```bash
mttctl node remove-favorite --dest 04e1c43b
mttctl node remove-favorite --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

---

## `node set-ignored`

Mark a node as ignored. Ignored nodes are filtered out of mesh activity on the local device.

```bash
mttctl node set-ignored --dest 04e1c43b
mttctl node set-ignored --to Pedro
```

| Option | Description |
|---|---|
| `--dest` | Node ID in hex (required unless `--to` is used) |
| `--to` | Node name (required unless `--dest` is used) |

---

## `node remove-ignored`

Remove a node from the ignored list.

```bash
mttctl node remove-ignored --dest 04e1c43b
mttctl node remove-ignored --to Pedro
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
mttctl node set-unmessageable

# Explicitly mark as unmessageable
mttctl node set-unmessageable true

# Restore as messageable
mttctl node set-unmessageable false
```

| Option | Description |
|---|---|
| `[VALUE]` | `true` to mark as unmessageable, `false` to mark as messageable (default: `true`) |
